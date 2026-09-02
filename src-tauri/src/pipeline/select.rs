//! Model call #2 and everything that keeps its answer honest.
//!
//! The model sees an ID-keyed list and may answer with IDs. Text it invents in a
//! `rewrite` is checked by `grounding`; anything that fails falls back to the stored
//! wording. An ID that was not on the list is dropped.

use super::grounding;
use super::plan::{PlanBullet, PlanRole, PlanSection, ResumePlan};
use super::skills::SkillIndex;
use super::strength;
use crate::domain::*;
use crate::error::Result;
use crate::llm::{self, client::LlmClient, schemas::ParsedJob};
use crate::render::tex::format_dates;
use std::collections::{HashMap, HashSet};
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

/// Fills every experience already on the page with the rest of its bullets.
///
/// The model picks one line at a time and for relevance alone, so it will happily return a
/// single bullet from each of eight experiences — a page that reads as a list of places
/// rather than as evidence about any of them. Nothing downstream can repair that: the plan
/// prints what was picked, `cap_bullets` only ever removes, and the fit loop's grow pass
/// spends a bench that a thin selection never filled.
///
/// So an entry that earned the page earns its lines. Every other active bullet under it
/// joins it verbatim, and the cap and the fit loop decide from there how many actually
/// print — three per experience, two per project, weakest cut first. Reads `all` rather
/// than the shortlist because the shortlist is the model's attention budget, not a printing
/// budget: an entry that got one seat on it would otherwise stay one line long. Nothing new
/// reaches the page this way — an experience the model passed over entirely stays off — and
/// every line added is stored text, so grounding has nothing to check.
fn deepen(chosen: &mut Vec<(Uuid, Chosen)>, all: &[Candidate]) {
    let printed: HashSet<Uuid> = chosen.iter().map(|(id, _)| *id).collect();
    let on_page: HashSet<Uuid> = all
        .iter()
        .filter(|c| printed.contains(&c.bullet_id))
        .map(|c| c.experience_id)
        .collect();

    for c in all {
        if !printed.contains(&c.bullet_id) && on_page.contains(&c.experience_id) {
            chosen.push((
                c.bullet_id,
                Chosen {
                    text: c.text.clone(),
                    was_reworded: false,
                    score: c.score,
                },
            ));
        }
    }
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
    all: &[Candidate],
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

    deepen(&mut chosen, all);
    let skills_line = index.line(selection.skills_line.iter().map(String::as_str));

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
    all: &[Candidate],
    vault: &[ExperienceDetail],
    profile: Option<Profile>,
    index: &SkillIndex,
    wanted: &[String],
) -> SelectionOutcome {
    let mut ranked: Vec<&Candidate> = shortlist.iter().collect();
    ranked.sort_by(|a, b| b.score.total_cmp(&a.score));
    let mut chosen: Vec<(Uuid, Chosen)> = ranked
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

    // Taking the top scorers spreads them across experiences exactly as the model does.
    deepen(&mut chosen, all);
    let skills_line = index.line(wanted.iter().map(String::as_str));

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
    let merit = strength::score_all(vault);
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
                role_id: role.role.id,
                // GPA rides on the degree line rather than getting its own template
                // variable: every layout already prints a role title, so this reaches both
                // built-ins and any template the user writes without either knowing about it.
                title: match role
                    .role
                    .gpa
                    .as_deref()
                    .map(str::trim)
                    .filter(|g| !g.is_empty())
                {
                    Some(gpa) => format!("{}, GPA: {}", role.role.title, gpa),
                    None => role.role.title.clone(),
                },
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
        // An experience is judged on its strongest line rather than the average of them:
        // the lines `deepen` adds always score below the one that earned the entry its
        // place, and a deeper entry must not lose that place for carrying them. Fit for the
        // posting when the bullets were scored, intrinsic merit when they were not — the
        // base resume leaves every bullet at 0.0, so `merit` is what ranks it.
        let fit: f32 = roles
            .iter()
            .flat_map(|r| r.bullets.iter())
            .map(|b| b.score)
            .fold(0.0_f32, f32::max);
        let section = PlanSection {
            experience_id: detail.experience.id,
            org: detail.experience.org_name.clone(),
            location: detail.experience.location.clone().unwrap_or_default(),
            tech: detail.experience.tech_line.clone().unwrap_or_default(),
            url: detail.experience.url.clone(),
            link_text: detail.experience.link_text.clone(),
            keep_empty,
            score: if fit > 0.0 {
                fit
            } else {
                merit.get(&detail.experience.id).copied().unwrap_or(0.0)
            },
            pinned: detail.experience.is_pinned,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn candidate(experience_id: Uuid, text: &str, score: f32) -> Candidate {
        Candidate {
            bullet_id: Uuid::new_v4(),
            text: text.into(),
            role_id: Uuid::new_v4(),
            role_title: "Intern".into(),
            end_date: None,
            experience_id,
            org_name: "Org".into(),
            skills: vec![],
            experience_skills: vec![],
            score,
        }
    }

    /// The failure this exists to prevent: a page of entries carrying one line each, because
    /// the model picked one bullet from each of them and nothing downstream adds lines.
    #[test]
    fn an_experience_on_the_page_is_filled_out_and_one_left_off_stays_off() {
        let (kept, ignored) = (Uuid::new_v4(), Uuid::new_v4());
        let shortlist = vec![
            candidate(kept, "the line the model picked", 4.0),
            candidate(kept, "another line from the same job", 2.5),
            candidate(kept, "a third line from the same job", 1.0),
            candidate(ignored, "a line from a job it passed over", 3.0),
        ];
        let mut chosen = vec![(
            shortlist[0].bullet_id,
            Chosen {
                text: shortlist[0].text.clone(),
                was_reworded: false,
                score: shortlist[0].score,
            },
        )];

        deepen(&mut chosen, &shortlist);

        let ids: Vec<Uuid> = chosen.iter().map(|(id, _)| *id).collect();
        assert_eq!(
            ids.len(),
            3,
            "the picked entry carries its own lines: {ids:?}"
        );
        assert!(ids.contains(&shortlist[1].bullet_id));
        assert!(ids.contains(&shortlist[2].bullet_id));
        assert!(
            !ids.contains(&shortlist[3].bullet_id),
            "an experience the model passed over must not appear"
        );
        assert!(
            chosen.iter().all(|(_, c)| !c.was_reworded),
            "a filled-in line is the stored wording, so nothing claims a rewrite"
        );
    }
}
