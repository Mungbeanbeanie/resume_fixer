//! The draft resume between selection and rendering.
//!
//! Every line here carries the `bullet_id` it came from. The renderer is fed from this
//! struct and from nothing else, so a line without a stored bullet behind it cannot exist.

use crate::domain::{Profile, UsedBullet};
use crate::render::tex::{ExperienceBlock, ProjectBlock, RenderInput, RoleBlock};
use uuid::Uuid;

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
