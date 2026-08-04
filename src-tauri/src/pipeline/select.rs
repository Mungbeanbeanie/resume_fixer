//! Model call #2 and everything that keeps its answer honest.
//!
//! The model sees an ID-keyed list and may answer with IDs. Text it invents in a
//! `rewrite` is checked by `grounding`; anything that fails falls back to the stored
//! wording. An ID that was not on the list is dropped.

use super::grounding;
use super::plan::{PlanBullet, PlanRole, PlanSection, ResumePlan};
use super::skills::SkillIndex;
use crate::domain::*;
use crate::error::Result;
use crate::llm::{self, client::LlmClient, schemas::ParsedJob};
use crate::render::tex::format_dates;
use std::collections::HashMap;
use uuid::Uuid;

/// Without a model, this many of the top-scoring bullets make a plausible page.
const FALLBACK_BULLETS: usize = 14;

pub struct SelectionOutcome {
    pub plan: ResumePlan,
    pub rejected: Vec<RejectedRewrite>,
}

/// One bullet the plan will print, already grounded.
struct Chosen {
    text: String,
    was_reworded: bool,
    score: f32,
}

fn candidate_list(shortlist: &[Candidate]) -> String {
    shortlist
        .iter()
        .map(|c| {
            format!(
                "{} | {} — {} | {}",
                c.bullet_id, c.org_name, c.role_title, c.text
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

/// Asks the model to pick and reword, then verifies every word of the answer.
// The stage genuinely needs all of it: the posting, the shortlist, the vault it maps back
// onto, and the skill index the grounding checks read.
#[allow(clippy::too_many_arguments)]
pub async fn run(
    llm: &dyn LlmClient,
    parsed: &ParsedJob,
    shortlist: &[Candidate],
    vault: &[ExperienceDetail],
    profile: Option<Profile>,
    index: &SkillIndex,
    all_skills: &[Skill],
    feedback: Option<&str>,
) -> Result<SelectionOutcome> {
    let job = serde_json::to_string_pretty(parsed).unwrap_or_default();
    let skills = all_skills
        .iter()
        .map(|s| s.name.as_str())
        .collect::<Vec<_>>()
        .join(", ");

    let selection = llm::select(llm, &job, &candidate_list(shortlist), &skills, feedback).await?;

    let by_id: HashMap<Uuid, &Candidate> = shortlist.iter().map(|c| (c.bullet_id, c)).collect();
    let tagged: HashMap<Uuid, Vec<String>> = shortlist
        .iter()
        .map(|c| {
            (
                c.bullet_id,
                c.skills.iter().map(|(s, _)| s.clone()).collect(),
            )
        })
        .collect();

    let mut chosen: Vec<(Uuid, Chosen)> = Vec::new();
    let mut rejected = Vec::new();

    for section in &selection.sections {
        for picked in &section.bullets {
            let Some(candidate) = by_id.get(&picked.id) else {
                tracing::warn!(
                    "model returned an id that was not on the list: {}",
                    picked.id
                );
                continue;
            };
            if chosen.iter().any(|(id, _)| *id == picked.id) {
                continue;
            }
            let empty = Vec::new();
            let slugs = tagged.get(&picked.id).unwrap_or(&empty);
            let (text, was_reworded) = match picked.rewrite.as_deref().map(str::trim) {
                Some(rewrite) if !rewrite.is_empty() && rewrite != candidate.text => {
                    match grounding::check(&candidate.text, rewrite, slugs, index) {
                        Ok(()) => (rewrite.to_string(), true),
                        Err(reason) => {
                            rejected.push(RejectedRewrite {
                                bullet_id: picked.id,
                                attempted: rewrite.to_string(),
                                reason: reason.to_string(),
                            });
                            (candidate.text.clone(), false)
                        }
                    }
                }
                _ => (candidate.text.clone(), false),
            };
            chosen.push((
                picked.id,
                Chosen {
                    text,
                    was_reworded,
                    score: candidate.score,
                },
            ));
        }
    }

    let skills_line = index
        .resolve_all(selection.skills_line.iter().map(String::as_str))
        .iter()
        .filter_map(|slug| index.display(slug).map(str::to_string))
        .collect();

    Ok(SelectionOutcome {
        plan: assemble(chosen, vault, profile, skills_line),
        rejected,
    })
}

/// The no-model path: highest-scoring bullets, verbatim.
///
/// Used when the model call fails. A worse resume is still a resume; a crash is not.
pub fn deterministic(
    shortlist: &[Candidate],
    vault: &[ExperienceDetail],
    profile: Option<Profile>,
    index: &SkillIndex,
    wanted: &[String],
) -> SelectionOutcome {
    let mut ranked: Vec<&Candidate> = shortlist.iter().collect();
    ranked.sort_by(|a, b| b.score.total_cmp(&a.score));
    let chosen = ranked
        .into_iter()
        .take(FALLBACK_BULLETS)
        .map(|c| {
            (
                c.bullet_id,
                Chosen {
                    text: c.text.clone(),
                    was_reworded: false,
                    score: c.score,
                },
            )
        })
        .collect();

    let skills_line = index
        .resolve_all(wanted.iter().map(String::as_str))
        .iter()
        .filter_map(|slug| index.display(slug).map(str::to_string))
        .collect();

    SelectionOutcome {
        plan: assemble(chosen, vault, profile, skills_line),
        rejected: Vec::new(),
    }
}

/// Every active bullet, verbatim — the base resume.
///
/// No posting, no model, so nothing is scored and nothing is reworded. Grounding does not
/// apply: the text printed is the text stored.
pub fn everything(
    vault: &[ExperienceDetail],
    profile: Option<Profile>,
    skills_line: Vec<String>,
) -> ResumePlan {
    let chosen = vault
        .iter()
        .flat_map(|e| e.roles.iter())
        .flat_map(|r| r.bullets.iter())
        .filter(|b| b.bullet.is_active)
        .map(|b| {
            (
                b.bullet.id,
                Chosen {
                    text: b.bullet.text.clone(),
                    was_reworded: false,
                    score: 0.0,
                },
            )
        })
        .collect();
    assemble(chosen, vault, profile, skills_line)
}

/// Lays chosen bullets back over the vault's structure.
///
/// Section and role order come from the vault, never from the model: the model chooses
/// content, the user's ordering decides layout. Education and certifications are always
/// printed — a resume without them is missing something the posting did not ask about.
fn assemble(
    chosen: Vec<(Uuid, Chosen)>,
    vault: &[ExperienceDetail],
    profile: Option<Profile>,
    skills_line: Vec<String>,
) -> ResumePlan {
    let picked: HashMap<Uuid, Chosen> = chosen.into_iter().collect();
    let mut plan = ResumePlan {
        profile,
        skills_line,
        ..Default::default()
    };

    for detail in vault {
        if !detail.experience.is_active {
            continue;
        }
        if detail.experience.kind == ExperienceKind::Certification {
            plan.certifications.push(detail.experience.org_name.clone());
            continue;
        }
        let is_education = detail.experience.kind == ExperienceKind::Education;
        // Activities are often a title and a date with nothing under them; an empty one is
        // still worth printing, unlike a job with no bullets.
        let keep_empty = is_education || detail.experience.kind == ExperienceKind::Activity;
        let mut roles = Vec::new();
        for role in &detail.roles {
            if !role.role.is_active {
                continue;
            }
            let mut bullets: Vec<PlanBullet> = role
                .bullets
                .iter()
                .filter(|b| b.bullet.is_active)
                .filter_map(|b| {
                    picked.get(&b.bullet.id).map(|c| PlanBullet {
                        bullet_id: b.bullet.id,
                        source_text: b.bullet.text.clone(),
                        text: c.text.clone(),
                        was_reworded: c.was_reworded,
                        score: c.score,
                    })
                })
                .collect();
            // Coursework is not job-tailored; if the model ignored it, print it anyway.
            if is_education && bullets.is_empty() {
                bullets = role
                    .bullets
                    .iter()
                    .filter(|b| b.bullet.is_active)
                    .map(|b| PlanBullet {
                        bullet_id: b.bullet.id,
                        source_text: b.bullet.text.clone(),
                        text: b.bullet.text.clone(),
                        was_reworded: false,
                        score: 0.0,
                    })
                    .collect();
            }
            if bullets.is_empty() && !keep_empty {
                continue;
            }
            roles.push(PlanRole {
                title: role.role.title.clone(),
                dates: format_dates(
                    role.role.start_date,
                    role.role.end_date,
                    role.role.date_override.as_deref(),
                ),
                bullets,
            });
        }
        if roles.is_empty() {
            continue;
        }
        let section = PlanSection {
            experience_id: detail.experience.id,
            org: detail.experience.org_name.clone(),
            location: detail.experience.location.clone().unwrap_or_default(),
            tech: detail.experience.tech_line.clone().unwrap_or_default(),
            roles,
        };
        match detail.experience.kind {
            ExperienceKind::Education => plan.education.push(section),
            ExperienceKind::Work => plan.experience.push(section),
            ExperienceKind::Project => plan.projects.push(section),
            ExperienceKind::Activity => plan.activities.push(section),
            ExperienceKind::Certification => unreachable!("handled above"),
        }
    }
    plan
}
