//! The draft resume between selection and rendering.
//!
//! Every line here carries the `bullet_id` it came from. The renderer is fed from this
//! struct and from nothing else, so a line without a stored bullet behind it cannot exist.

use crate::domain::{BulletEdit, Profile, UsedBullet};
use crate::render::tex::{ExperienceBlock, ProjectBlock, RenderInput, RoleBlock};
use std::collections::HashSet;
use uuid::Uuid;

/// The most bullets one experience prints.
///
/// Past three, an entry stops being the strongest evidence for a job and becomes a diary.
/// Capping here rather than in the fit loop means the cut is made on merit — the lowest
/// scoring lines of that experience — instead of on whatever happened to overflow a page.
const MAX_BULLETS_PER_EXPERIENCE: usize = 3;

#[derive(Debug, Clone)]
pub struct PlanBullet {
    pub bullet_id: Uuid,
    pub source_text: String,
    pub text: String,
    pub was_reworded: bool,
    pub score: f32,
}

#[derive(Debug, Clone)]
pub struct PlanRole {
    pub title: String,
    pub dates: String,
    pub bullets: Vec<PlanBullet>,
}

#[derive(Debug, Clone)]
pub struct PlanSection {
    pub experience_id: Uuid,
    pub org: String,
    pub location: String,
    pub tech: String,
    /// Education and activities print a heading that carries the fact by itself — a degree,
    /// a club. Everywhere else a role with no bullets left under it is dead weight.
    pub keep_empty: bool,
    /// How well this experience earns its space: fit for the posting on a tailored resume,
    /// intrinsic merit on the base one. Only `retire_weakest` reads it.
    pub score: f32,
    /// The user said this one always prints. `retire_weakest` skips it.
    pub pinned: bool,
    pub roles: Vec<PlanRole>,
}

impl PlanSection {
    fn bullet_count(&self) -> usize {
        self.roles.iter().map(|r| r.bullets.len()).sum()
    }
}

#[derive(Debug, Clone, Default)]
pub struct ResumePlan {
    pub profile: Option<Profile>,
    pub education: Vec<PlanSection>,
    pub experience: Vec<PlanSection>,
    pub projects: Vec<PlanSection>,
    pub activities: Vec<PlanSection>,
    pub certifications: Vec<String>,
    pub skills_line: Vec<String>,
}

fn to_block(s: &PlanSection) -> ExperienceBlock {
    ExperienceBlock {
        org: s.org.clone(),
        location: s.location.clone(),
        roles: s
            .roles
            .iter()
            .map(|r| RoleBlock {
                title: r.title.clone(),
                dates: r.dates.clone(),
                bullets: r.bullets.iter().map(|b| b.text.clone()).collect(),
            })
            .collect(),
    }
}

impl ResumePlan {
    /// The template's view of the plan: text only, ready to be escaped and rendered.
    pub fn to_render_input(&self) -> RenderInput {
        RenderInput {
            profile: self.profile.clone(),
            education: self.education.iter().map(to_block).collect(),
            skills_line: self.skills_line.join(", "),
            experience: self.experience.iter().map(to_block).collect(),
            projects: self
                .projects
                .iter()
                .map(|p| ProjectBlock {
                    name: p.org.clone(),
                    tech: p.tech.clone(),
                    dates: p.roles.first().map(|r| r.dates.clone()).unwrap_or_default(),
                    bullets: p
                        .roles
                        .iter()
                        .flat_map(|r| r.bullets.iter())
                        .map(|b| b.text.clone())
                        .collect(),
                })
                .collect(),
            activities: self.activities.iter().map(to_block).collect(),
            certifications: self.certifications.clone(),
        }
    }

    /// Drops the sections the template cannot print.
    ///
    /// A template prints a section only if it names the variable, so the source text is
    /// the authority. Dropping here rather than at render time keeps provenance honest:
    /// a bullet that was never printed is never recorded as used, and the fit loop never
    /// gives up a bullet to make room for a section that is not there.
    pub fn prune_for(&mut self, template: &str) {
        if !template.contains("activities") {
            self.activities.clear();
        }
    }

    /// Provenance rows, in printed order.
    pub fn used_bullets(&self) -> Vec<UsedBullet> {
        self.sections()
            .flat_map(|s| {
                s.roles.iter().flat_map(move |r| {
                    r.bullets.iter().map(move |b| UsedBullet {
                        bullet_id: b.bullet_id,
                        source_text: b.source_text.clone(),
                        rendered_text: b.text.clone(),
                        was_reworded: b.was_reworded,
                        org_name: s.org.clone(),
                    })
                })
            })
            .collect()
    }

    fn sections(&self) -> impl Iterator<Item = &PlanSection> {
        self.education
            .iter()
            .chain(self.experience.iter())
            .chain(self.projects.iter())
            .chain(self.activities.iter())
    }

    pub fn bullet_count(&self) -> usize {
        self.sections().map(PlanSection::bullet_count).sum()
    }

    /// Keeps at most `MAX_BULLETS_PER_EXPERIENCE` bullets under each experience, dropping
    /// the lowest scoring first, and clears out whatever that empties.
    ///
    /// Position order is preserved: the cut decides *which* bullets print, never the order
    /// they print in — that stays the user's, from the vault. On the base resume nothing is
    /// scored, so every bullet ties and the stable sort leaves the first three in vault
    /// order.
    pub fn cap_bullets(&mut self) {
        for sections in self.all_sections_mut() {
            for section in sections.iter_mut() {
                if section.bullet_count() <= MAX_BULLETS_PER_EXPERIENCE {
                    continue;
                }
                let mut ranked: Vec<(usize, usize, f32)> = section
                    .roles
                    .iter()
                    .enumerate()
                    .flat_map(|(ri, r)| {
                        r.bullets
                            .iter()
                            .enumerate()
                            .map(move |(bi, b)| (ri, bi, b.score))
                    })
                    .collect();
                ranked.sort_by(|a, b| b.2.total_cmp(&a.2));
                let keep: HashSet<(usize, usize)> = ranked
                    .into_iter()
                    .take(MAX_BULLETS_PER_EXPERIENCE)
                    .map(|(ri, bi, _)| (ri, bi))
                    .collect();

                for (ri, role) in section.roles.iter_mut().enumerate() {
                    let mut bi = 0;
                    role.bullets.retain(|_| {
                        let keeping = keep.contains(&(ri, bi));
                        bi += 1;
                        keeping
                    });
                }
            }
        }
        self.prune_empty();
    }

    /// Drops roles left with nothing under them, then sections left with no roles.
    ///
    /// A heading whose bullets all went away prints as a title floating over white space.
    /// Education and activities are exempt: their heading is the content.
    fn prune_empty(&mut self) {
        for sections in self.all_sections_mut() {
            for section in sections.iter_mut() {
                if !section.keep_empty {
                    section.roles.retain(|r| !r.bullets.is_empty());
                }
            }
            sections.retain(|s| !s.roles.is_empty());
        }
    }

    fn all_sections_mut(&mut self) -> [&mut Vec<PlanSection>; 4] {
        [
            &mut self.education,
            &mut self.experience,
            &mut self.projects,
            &mut self.activities,
        ]
    }

    /// Applies the user's own edits in place: reworded lines take the new text, lines with
    /// `keep: false` leave this resume.
    ///
    /// `source_text` is never touched — it is what the vault holds, and the gap between it
    /// and `text` is the record of what the user changed. `was_reworded` goes false because
    /// the model did not author this line: grounding has nothing to check, and the library
    /// must not credit the model with a line the user wrote. An edit naming a bullet the
    /// plan does not carry is ignored rather than an error; the draft it described is gone.
    pub fn apply_edits(&mut self, edits: &[BulletEdit]) {
        for section in self.all_sections_mut().into_iter().flatten() {
            for role in section.roles.iter_mut() {
                role.bullets.retain(|b| {
                    edits
                        .iter()
                        .find(|e| e.bullet_id == b.bullet_id)
                        .is_none_or(|e| e.keep)
                });
                for bullet in role.bullets.iter_mut() {
                    let Some(edit) = edits.iter().find(|e| e.bullet_id == bullet.bullet_id) else {
                        continue;
                    };
                    let text = edit.text.trim();
                    if text.is_empty() || text == bullet.text {
                        continue;
                    }
                    bullet.text = text.to_string();
                    bullet.was_reworded = false;
                }
            }
        }
        self.prune_empty();
    }

    /// Retires the lowest-scoring experience whole, and returns its name.
    ///
    /// Tried before thinning anyone's bullets: three entries carrying three lines each say
    /// more than eight carrying one, so the page is bought by dropping the weakest evidence
    /// entirely rather than by starving every entry equally.
    ///
    /// Never touches education — a degree is not optional — and never a pinned entry. Keeps
    /// the last remaining experience whatever it scores: a resume of nothing but a header
    /// is not a shorter resume.
    ///
    /// Section rank decides before score does: activities are given up first, then projects,
    /// and paid work only when nothing else is left. Score alone would cut the internship and
    /// keep the side project, because a personal repo is far easier to pack with numbers than
    /// a job whose real work is under an NDA.
    pub fn retire_weakest(&mut self) -> Option<String> {
        if self.experience.len() + self.projects.len() + self.activities.len() <= 1 {
            return None;
        }

        for tier in [
            &mut self.activities,
            &mut self.projects,
            &mut self.experience,
        ] {
            let weakest = tier
                .iter()
                .enumerate()
                .filter(|(_, s)| !s.pinned)
                .min_by(|a, b| a.1.score.total_cmp(&b.1.score))
                .map(|(i, _)| i);
            if let Some(i) = weakest {
                return Some(tier.remove(i).org);
            }
        }
        None
    }

    /// Drops the weakest bullet from the section carrying the most, and returns it.
    ///
    /// Rejects the drop when it would empty an experience: a job with no bullets under it
    /// reads as padding, so the fit loop stops rather than produce one. `None` means
    /// nothing further can be given up.
    pub fn drop_lowest(&mut self) -> Option<PlanBullet> {
        let all: Vec<&mut PlanSection> = self
            .education
            .iter_mut()
            .chain(self.experience.iter_mut())
            .chain(self.projects.iter_mut())
            .chain(self.activities.iter_mut())
            .filter(|s| s.bullet_count() > 1)
            .collect();

        let section = all.into_iter().max_by_key(|s| s.bullet_count())?;
        let (role_i, bullet_i) = section
            .roles
            .iter()
            .enumerate()
            .flat_map(|(ri, r)| {
                r.bullets
                    .iter()
                    .enumerate()
                    .map(move |(bi, b)| (ri, bi, b.score))
            })
            .min_by(|a, b| a.2.total_cmp(&b.2))
            .map(|(ri, bi, _)| (ri, bi))?;

        let dropped = section.roles[role_i].bullets.remove(bullet_i);
        self.prune_empty();
        Some(dropped)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bullet(score: f32) -> PlanBullet {
        PlanBullet {
            bullet_id: Uuid::new_v4(),
            source_text: "source".into(),
            text: format!("bullet {score}"),
            was_reworded: false,
            score,
        }
    }

    fn section(org: &str, scores: &[f32]) -> PlanSection {
        PlanSection {
            experience_id: Uuid::new_v4(),
            org: org.into(),
            location: "Remote".into(),
            tech: String::new(),
            keep_empty: false,
            score: 0.0,
            pinned: false,
            roles: vec![PlanRole {
                title: "Intern".into(),
                dates: "2026".into(),
                bullets: scores.iter().map(|s| bullet(*s)).collect(),
            }],
        }
    }

    #[test]
    fn the_weakest_bullet_of_the_biggest_section_goes_first() {
        let mut plan = ResumePlan {
            experience: vec![section("Big", &[0.9, 0.2, 0.5]), section("Small", &[0.1])],
            ..Default::default()
        };
        let dropped = plan.drop_lowest().unwrap();
        assert_eq!(dropped.text, "bullet 0.2");
        assert_eq!(plan.bullet_count(), 3);
    }

    #[test]
    fn an_experience_is_never_emptied() {
        let mut plan = ResumePlan {
            experience: vec![section("A", &[0.1]), section("B", &[0.2])],
            ..Default::default()
        };
        assert!(plan.drop_lowest().is_none());
        assert_eq!(plan.bullet_count(), 2);
    }

    #[test]
    fn a_template_without_activities_neither_prints_nor_records_them() {
        let mut plan = ResumePlan {
            experience: vec![section("A", &[0.1])],
            activities: vec![section("Ice Hockey", &[0.2])],
            ..Default::default()
        };
        plan.prune_for("\\section{Experience} {% for e in experience %}");
        assert!(plan.activities.is_empty());
        assert_eq!(plan.used_bullets().len(), 1);

        let mut plan = ResumePlan {
            activities: vec![section("Ice Hockey", &[0.2])],
            ..Default::default()
        };
        plan.prune_for("{% if activities %}\\section{Activities}{% endif %}");
        assert_eq!(plan.used_bullets().len(), 1);
    }

    fn scored(org: &str, score: f32, pinned: bool) -> PlanSection {
        PlanSection {
            score,
            pinned,
            ..section(org, &[0.5, 0.5])
        }
    }

    #[test]
    fn the_weakest_experience_goes_first_and_a_pin_survives_it() {
        let mut plan = ResumePlan {
            education: vec![scored("A University", 0.0, false)],
            experience: vec![
                scored("Strong Co", 2.4, false),
                scored("Weak Co", 0.2, false),
                scored("Pinned Co", 0.1, true),
            ],
            ..Default::default()
        };

        assert_eq!(plan.retire_weakest(), Some("Weak Co".to_string()));
        assert_eq!(plan.retire_weakest(), Some("Strong Co".to_string()));
        assert_eq!(
            plan.retire_weakest(),
            None,
            "the last experience stays whatever it scores"
        );
        assert_eq!(plan.education.len(), 1, "education is never retired");
        assert_eq!(plan.experience[0].org, "Pinned Co");
    }

    #[test]
    fn a_side_project_is_given_up_before_a_job_that_scores_worse() {
        let mut plan = ResumePlan {
            experience: vec![scored("Rajant Health", 0.3, false)],
            projects: vec![scored("Car Simulation", 2.9, false)],
            activities: vec![scored("Ice Hockey", 2.0, false)],
            ..Default::default()
        };

        // Every one of these outscores the internship, and every one goes first anyway.
        assert_eq!(plan.retire_weakest(), Some("Ice Hockey".to_string()));
        assert_eq!(plan.retire_weakest(), Some("Car Simulation".to_string()));
        assert_eq!(plan.experience[0].org, "Rajant Health");
        assert_eq!(plan.retire_weakest(), None, "the last one stands");
    }

    #[test]
    fn retiring_stops_rather_than_empty_a_resume_of_pins() {
        let mut plan = ResumePlan {
            experience: vec![scored("Pinned A", 0.1, true), scored("Pinned B", 0.2, true)],
            ..Default::default()
        };
        assert_eq!(plan.retire_weakest(), None, "nothing is eligible");
        assert_eq!(plan.experience.len(), 2);
    }

    #[test]
    fn an_experience_prints_at_most_three_bullets_its_best_ones() {
        let mut plan = ResumePlan {
            experience: vec![section("Big", &[0.1, 0.9, 0.4, 0.8, 0.2])],
            ..Default::default()
        };
        plan.cap_bullets();
        let printed: Vec<String> = plan
            .used_bullets()
            .into_iter()
            .map(|b| b.rendered_text)
            .collect();
        assert_eq!(printed, ["bullet 0.9", "bullet 0.4", "bullet 0.8"]);
    }

    #[test]
    fn the_cap_spans_an_experiences_roles_and_clears_the_ones_it_empties() {
        let mut plan = ResumePlan {
            experience: vec![PlanSection {
                experience_id: Uuid::new_v4(),
                org: "Rajant Health".into(),
                location: "Remote".into(),
                tech: String::new(),
                keep_empty: false,
                score: 0.0,
                pinned: false,
                roles: vec![
                    PlanRole {
                        title: "Computer Engineering Intern".into(),
                        dates: "2025".into(),
                        bullets: vec![bullet(0.1), bullet(0.2)],
                    },
                    PlanRole {
                        title: "Software Engineering Intern".into(),
                        dates: "2026".into(),
                        bullets: vec![bullet(0.9), bullet(0.8), bullet(0.7)],
                    },
                ],
            }],
            ..Default::default()
        };
        plan.cap_bullets();
        assert_eq!(plan.bullet_count(), 3, "three across the whole experience");
        assert_eq!(plan.experience[0].roles.len(), 1, "the emptied role goes");
        assert_eq!(
            plan.experience[0].roles[0].title,
            "Software Engineering Intern"
        );
    }

    #[test]
    fn education_keeps_a_heading_the_cap_left_bare() {
        let mut plan = ResumePlan {
            education: vec![PlanSection {
                keep_empty: true,
                ..section("A University", &[])
            }],
            ..Default::default()
        };
        plan.cap_bullets();
        assert_eq!(
            plan.education.len(),
            1,
            "a degree is the content, not its bullets"
        );
    }

    #[test]
    fn a_user_edit_rewrites_the_line_without_touching_its_source() {
        let mut plan = ResumePlan {
            experience: vec![section("A", &[0.1, 0.2])],
            ..Default::default()
        };
        let kept = plan.experience[0].roles[0].bullets[0].bullet_id;
        let dropped = plan.experience[0].roles[0].bullets[1].bullet_id;
        plan.experience[0].roles[0].bullets[0].was_reworded = true;

        plan.apply_edits(&[
            BulletEdit {
                bullet_id: kept,
                text: "  Shipped the thing, 25% faster.  ".into(),
                keep: true,
            },
            BulletEdit {
                bullet_id: dropped,
                text: "does not matter".into(),
                keep: false,
            },
            BulletEdit {
                bullet_id: Uuid::new_v4(),
                text: "belongs to another draft".into(),
                keep: false,
            },
        ]);

        let used = plan.used_bullets();
        assert_eq!(used.len(), 1, "keep: false drops the line");
        assert_eq!(used[0].rendered_text, "Shipped the thing, 25% faster.");
        assert_eq!(used[0].source_text, "source", "the vault text is untouched");
        assert!(
            !used[0].was_reworded,
            "the user wrote this line, not the model"
        );
    }

    #[test]
    fn provenance_covers_every_printed_line() {
        let plan = ResumePlan {
            experience: vec![section("A", &[0.1, 0.2])],
            projects: vec![section("P", &[0.3])],
            ..Default::default()
        };
        let input = plan.to_render_input();
        let printed: usize = input
            .experience
            .iter()
            .flat_map(|e| e.roles.iter())
            .map(|r| r.bullets.len())
            .sum::<usize>()
            + input
                .projects
                .iter()
                .map(|p| p.bullets.len())
                .sum::<usize>();
        assert_eq!(plan.used_bullets().len(), printed);
    }
}
