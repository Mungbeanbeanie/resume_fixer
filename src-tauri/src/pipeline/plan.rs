//! The draft resume between selection and rendering.
//!
//! Every line here carries the `bullet_id` it came from. The renderer is fed from this
//! struct and from nothing else, so a line without a stored bullet behind it cannot exist.

use crate::domain::{BulletEdit, ExperienceDetail, Profile, UsedBullet};
use crate::render::tex::{format_dates, ExperienceBlock, ProjectBlock, RenderInput, RoleBlock};
use std::collections::HashSet;
use uuid::Uuid;

/// The most bullets one experience prints.
///
/// Past three, an entry stops being the strongest evidence for a job and becomes a diary.
/// Capping here rather than in the fit loop means the cut is made on merit — the lowest
/// scoring lines of that experience — instead of on whatever happened to overflow a page.
const MAX_BULLETS_PER_EXPERIENCE: usize = 3;

/// A project starts at two.
///
/// Three projects carrying strong lines say more than two padded out to three each, and a
/// project's third-best line is usually its weakest — the personal repo is easy to write a
/// third bullet about and hard to write a good one. Nothing is lost by starting lean: the
/// fit loop's grow pass puts a third back when the page has the room for it.
const MAX_BULLETS_PER_PROJECT: usize = 2;

/// How strong a project's third bullet has to be to print without waiting for the grow
/// pass: within this much of that project's own best line. Judged against the project
/// rather than the resume so a weak entry cannot buy a third slot by being weak evenly.
const PROJECT_THIRD_RATIO: f32 = 0.8;

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
    pub role_id: Uuid,
    pub title: String,
    pub dates: String,
    pub bullets: Vec<PlanBullet>,
}

/// A bullet set aside by the cap or the fit loop, with everything needed to put it back
/// where it came from.
#[derive(Debug, Clone)]
pub(super) struct Benched {
    experience_id: Uuid,
    role_id: Uuid,
    role_title: String,
    dates: String,
    /// Its index within the role, so restoring keeps the vault's ordering.
    at: usize,
    bullet: PlanBullet,
}

/// An experience the fit loop retired whole, with the list and position it came from.
#[derive(Debug, Clone)]
pub(super) struct Retired {
    /// Index into `all_sections_mut`.
    tier: usize,
    at: usize,
    section: PlanSection,
}

#[derive(Debug, Clone)]
pub struct PlanSection {
    pub experience_id: Uuid,
    pub org: String,
    pub location: String,
    pub tech: String,
    /// Where a project lives. The Projects section prints it in place of the dates.
    pub url: Option<String>,
    /// What that link reads as; blank falls back to the shortened URL.
    pub link_text: Option<String>,
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
    /// What the cap and the fit loop took, so the grow pass can offer it back. Never
    /// rendered and never counted — `to_render_input`, `used_bullets` and `bullet_count`
    /// all walk the named sections.
    pub(super) bench: Vec<Benched>,
    pub(super) retired: Vec<Retired>,
}

/// Where a vault bullet belongs on the page.
struct Found {
    experience_id: Uuid,
    role_id: Uuid,
    role_title: String,
    dates: String,
    at: usize,
    bullet: PlanBullet,
}

/// Locates an active vault bullet and dresses it for the plan.
///
/// `at` counts only the active bullets of its role, which is what the plan prints, so a
/// restored line lands in the order the vault has it rather than at the end.
fn find_in_vault(vault: &[ExperienceDetail], bullet_id: Uuid) -> Option<Found> {
    for detail in vault {
        for role in &detail.roles {
            let active = || role.bullets.iter().filter(|b| b.bullet.is_active);
            let Some(at) = active().position(|b| b.bullet.id == bullet_id) else {
                continue;
            };
            let found = active().nth(at).expect("just found by position");
            return Some(Found {
                experience_id: detail.experience.id,
                role_id: role.role.id,
                role_title: role.role.title.clone(),
                dates: format_dates(
                    role.role.start_date,
                    role.role.end_date,
                    role.role.date_override.as_deref(),
                ),
                at,
                bullet: PlanBullet {
                    bullet_id,
                    source_text: found.bullet.text.clone(),
                    text: found.bullet.text.clone(),
                    was_reworded: false,
                    score: 0.0,
                },
            });
        }
    }
    None
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
                    url: p.url.clone(),
                    link_text: p.link_text.clone(),
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

    /// Keeps at most `MAX_BULLETS_PER_EXPERIENCE` bullets under each experience — two under
    /// a project — dropping the lowest scoring first, and clears out whatever that empties.
    ///
    /// Position order is preserved: the cut decides *which* bullets print, never the order
    /// they print in — that stays the user's, from the vault. On the base resume nothing is
    /// scored, so every bullet ties and the stable sort leaves the first ones in vault
    /// order. What is cut goes on the bench rather than the floor, so the fit loop can put
    /// it back if the finished page has room.
    pub fn cap_bullets(&mut self) {
        let mut benched: Vec<Benched> = Vec::new();
        // Projects are the third bucket; see `all_sections_mut`.
        for (tier, sections) in self.all_sections_mut().into_iter().enumerate() {
            let is_project = tier == 2;
            for section in sections.iter_mut() {
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

                let limit = if is_project {
                    // The third prints straight away only when it stands with the best of
                    // its own project; otherwise it waits for the grow pass.
                    let strong_third = ranked.len() > MAX_BULLETS_PER_PROJECT
                        && ranked[MAX_BULLETS_PER_PROJECT].2 > 0.0
                        && ranked[MAX_BULLETS_PER_PROJECT].2 >= PROJECT_THIRD_RATIO * ranked[0].2;
                    if strong_third {
                        MAX_BULLETS_PER_EXPERIENCE
                    } else {
                        MAX_BULLETS_PER_PROJECT
                    }
                } else {
                    MAX_BULLETS_PER_EXPERIENCE
                };
                if ranked.len() <= limit {
                    continue;
                }

                let keep: HashSet<(usize, usize)> = ranked
                    .into_iter()
                    .take(limit)
                    .map(|(ri, bi, _)| (ri, bi))
                    .collect();

                let experience_id = section.experience_id;
                for (ri, role) in section.roles.iter_mut().enumerate() {
                    let (role_id, title, dates) =
                        (role.role_id, role.title.clone(), role.dates.clone());
                    let mut bi = 0;
                    role.bullets.retain(|b| {
                        let keeping = keep.contains(&(ri, bi));
                        if !keeping {
                            benched.push(Benched {
                                experience_id,
                                role_id,
                                role_title: title.clone(),
                                dates: dates.clone(),
                                at: bi,
                                bullet: b.clone(),
                            });
                        }
                        bi += 1;
                        keeping
                    });
                }
            }
        }
        self.bench.append(&mut benched);
        self.prune_empty();
    }

    /// Puts a bullet under its experience, recreating the role if the cap emptied it.
    ///
    /// Returns false when the experience is not on the resume at all — a retired entry has
    /// to come back whole, through `restore_section`, before its lines mean anything.
    /// Roles are matched by id, not by title: the education heading carries a GPA the vault
    /// row does not.
    fn insert_bullet(
        &mut self,
        experience_id: Uuid,
        role_id: Uuid,
        role_title: &str,
        dates: &str,
        at: usize,
        bullet: PlanBullet,
    ) -> bool {
        let Some(section) = self
            .all_sections_mut()
            .into_iter()
            .flatten()
            .find(|s| s.experience_id == experience_id)
        else {
            return false;
        };
        let role = match section.roles.iter_mut().position(|r| r.role_id == role_id) {
            Some(i) => &mut section.roles[i],
            None => {
                section.roles.push(PlanRole {
                    role_id,
                    title: role_title.to_string(),
                    dates: dates.to_string(),
                    bullets: Vec::new(),
                });
                section.roles.last_mut().expect("just pushed")
            }
        };
        if role.bullets.iter().any(|b| b.bullet_id == bullet.bullet_id) {
            return false;
        }
        let at = at.min(role.bullets.len());
        role.bullets.insert(at, bullet);
        true
    }

    /// True when this bullet is already printed.
    fn holds(&self, bullet_id: Uuid) -> bool {
        self.sections()
            .flat_map(|s| s.roles.iter())
            .flat_map(|r| r.bullets.iter())
            .any(|b| b.bullet_id == bullet_id)
    }

    /// How many bullets one experience is printing. Zero for one that is not on the page.
    fn printed_under(&self, experience_id: Uuid) -> usize {
        self.sections()
            .find(|s| s.experience_id == experience_id)
            .map_or(0, PlanSection::bullet_count)
    }

    /// Puts back the strongest benched bullet whose experience is still on the resume and
    /// still under the cap.
    ///
    /// The cap is the ceiling here too, not just at `cap_bullets`: a project may grow from
    /// two lines to three because it started lean, but nothing reaches four. Room at the
    /// bottom of the page is spent on another experience, never on a fourth line about one
    /// that already said its piece.
    ///
    /// Returns false when there is nothing left to restore.
    pub fn restore_bullet(&mut self) -> bool {
        loop {
            let Some(i) = self
                .bench
                .iter()
                .enumerate()
                .filter(|(_, b)| {
                    !self.holds(b.bullet.bullet_id)
                        && self.printed_under(b.experience_id) < MAX_BULLETS_PER_EXPERIENCE
                })
                .max_by(|a, b| a.1.bullet.score.total_cmp(&b.1.bullet.score))
                .map(|(i, _)| i)
            else {
                return false;
            };
            let b = self.bench.remove(i);
            if self.insert_bullet(
                b.experience_id,
                b.role_id,
                &b.role_title,
                &b.dates,
                b.at,
                b.bullet,
            ) {
                return true;
            }
            // Its experience is retired; drop it and look at the next one.
        }
    }

    /// Un-retires the strongest experience the fit loop gave up, back where it was.
    ///
    /// Tried before `restore_bullet`: another entry says more than a third line on one that
    /// is already there.
    pub fn restore_section(&mut self) -> bool {
        let Some(i) = self
            .retired
            .iter()
            .enumerate()
            .max_by(|a, b| a.1.section.score.total_cmp(&b.1.section.score))
            .map(|(i, _)| i)
        else {
            return false;
        };
        let r = self.retired.remove(i);
        let buckets = self.all_sections_mut();
        let at = r.at.min(buckets[r.tier].len());
        buckets[r.tier].insert(at, r.section);
        true
    }

    /// The experiences still retired, by org name, for the UI to name.
    pub fn retired_names(&self) -> Vec<String> {
        self.retired.iter().map(|r| r.section.org.clone()).collect()
    }

    /// Marks every section as one the fit loop may not retire.
    ///
    /// A hand-built resume is exactly the entries the user picked, so the loop thins their
    /// bullets to reach one page and reports an honest second page when it cannot — it never
    /// drops an entry that was chosen by name.
    pub fn pin_all(&mut self) {
        for section in self.all_sections_mut().into_iter().flatten() {
            section.pinned = true;
        }
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
    /// `keep: false` leave this resume, and a kept line the plan does not carry is fetched
    /// from the vault and printed.
    ///
    /// `source_text` is never touched — it is what the vault holds, and the gap between it
    /// and `text` is the record of what the user changed. `was_reworded` goes false because
    /// the model did not author this line: grounding has nothing to check, and the library
    /// must not credit the model with a line the user wrote. Added lines are the stored text
    /// verbatim for the same reason. An edit naming a bullet that is in neither the plan nor
    /// the vault is ignored rather than an error; the draft it described is gone. So is one
    /// naming an experience this resume left off — that has to come back whole.
    pub fn apply_edits(&mut self, vault: &[ExperienceDetail], edits: &[BulletEdit]) {
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

        for edit in edits.iter().filter(|e| e.keep) {
            if self.holds(edit.bullet_id) {
                continue;
            }
            let Some(found) = find_in_vault(vault, edit.bullet_id) else {
                continue;
            };
            self.insert_bullet(
                found.experience_id,
                found.role_id,
                &found.role_title,
                &found.dates,
                found.at,
                found.bullet,
            );
            // It is printed now, so it must not also sit on the bench waiting to be
            // restored a second time.
            self.bench.retain(|b| b.bullet.bullet_id != edit.bullet_id);
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

        // `tier` is the index into `all_sections_mut`, so a restore puts it back in the
        // list it came from: activities 3, projects 2, experience 1.
        for tier in [3usize, 2, 1] {
            let mut buckets = self.all_sections_mut();
            let list = &mut buckets[tier];
            let weakest = list
                .iter()
                .enumerate()
                .filter(|(_, s)| !s.pinned)
                .min_by(|a, b| a.1.score.total_cmp(&b.1.score))
                .map(|(i, _)| i);
            if let Some(i) = weakest {
                let section = list.remove(i);
                let org = section.org.clone();
                self.retired.push(Retired {
                    tier,
                    at: i,
                    section,
                });
                return Some(org);
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

        let experience_id = section.experience_id;
        let role = &mut section.roles[role_i];
        let dropped = role.bullets.remove(bullet_i);
        let benched = Benched {
            experience_id,
            role_id: role.role_id,
            role_title: role.title.clone(),
            dates: role.dates.clone(),
            at: bullet_i,
            bullet: dropped.clone(),
        };
        self.bench.push(benched);
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
            url: None,
            link_text: None,
            keep_empty: false,
            score: 0.0,
            pinned: false,
            roles: vec![PlanRole {
                role_id: Uuid::new_v4(),
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

    /// What the Build tab rests on: after `pin_all` the fit loop has nothing left to retire,
    /// so it thins bullets to reach one page and reports a second page rather than dropping
    /// an entry the user picked by name.
    #[test]
    fn pin_all_leaves_the_fit_loop_nothing_to_retire() {
        let mut plan = ResumePlan {
            experience: vec![
                scored("Strong Co", 2.4, false),
                scored("Weak Co", 0.2, false),
            ],
            projects: vec![scored("A Side Project", 0.5, false)],
            ..Default::default()
        };

        plan.pin_all();

        assert_eq!(plan.retire_weakest(), None);
        assert_eq!(plan.experience.len(), 2);
        assert_eq!(plan.projects.len(), 1);
        assert!(plan.retired_names().is_empty());
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
                url: None,
                link_text: None,
                keep_empty: false,
                score: 0.0,
                pinned: false,
                roles: vec![
                    PlanRole {
                        role_id: Uuid::new_v4(),
                        title: "Computer Engineering Intern".into(),
                        dates: "2025".into(),
                        bullets: vec![bullet(0.1), bullet(0.2)],
                    },
                    PlanRole {
                        role_id: Uuid::new_v4(),
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

        plan.apply_edits(
            &[],
            &[
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
            ],
        );

        let used = plan.used_bullets();
        assert_eq!(used.len(), 1, "keep: false drops the line");
        assert_eq!(used[0].rendered_text, "Shipped the thing, 25% faster.");
        assert_eq!(used[0].source_text, "source", "the vault text is untouched");
        assert!(
            !used[0].was_reworded,
            "the user wrote this line, not the model"
        );
    }

    /// Vault fixture whose ids line up with a plan section, so `apply_edits` can find a
    /// bullet the plan does not carry.
    fn vault_for(section: &PlanSection, unused: &[(&str, bool)]) -> Vec<ExperienceDetail> {
        use crate::domain::*;
        let role = &section.roles[0];
        let printed = role.bullets.iter().map(|b| (b.bullet_id, b.text.clone()));
        let extra = unused
            .iter()
            .map(|(t, active)| (Uuid::new_v4(), t.to_string(), *active));
        vec![ExperienceDetail {
            experience: Experience {
                id: section.experience_id,
                kind: ExperienceKind::Work,
                org_name: section.org.clone(),
                location: None,
                url: None,
                link_text: None,
                tech_line: None,
                display_order: 0,
                is_active: true,
                is_pinned: false,
            },
            skills: vec![],
            roles: vec![RoleDetail {
                role: Role {
                    id: role.role_id,
                    experience_id: section.experience_id,
                    title: role.title.clone(),
                    location: None,
                    start_date: chrono::NaiveDate::from_ymd_opt(2026, 1, 1),
                    end_date: None,
                    date_override: Some(role.dates.clone()),
                    gpa: None,
                    display_order: 0,
                    is_active: true,
                },
                bullets: printed
                    .map(|(id, text)| (id, text, true))
                    .chain(extra)
                    .map(|(id, text, is_active)| BulletDetail {
                        bullet: Bullet {
                            id,
                            role_id: role.role_id,
                            text,
                            display_order: 0,
                            is_active,
                        },
                        skills: vec![],
                        variants: vec![],
                    })
                    .collect(),
            }],
        }]
    }

    #[test]
    fn a_kept_edit_for_a_bullet_the_plan_dropped_puts_it_back_verbatim() {
        let s = section("Rajant Health", &[0.9]);
        let vault = vault_for(
            &s,
            &[("Rewrote the ingest path, cutting latency 30%.", true)],
        );
        let added = vault[0].roles[0].bullets[1].bullet.id;
        let mut plan = ResumePlan {
            experience: vec![s],
            ..Default::default()
        };

        plan.apply_edits(
            &vault,
            &[BulletEdit {
                bullet_id: added,
                // The UI sends the vault text; the plan must print the stored wording.
                text: "Rewrote the ingest path, cutting latency 30%.".into(),
                keep: true,
            }],
        );

        let used = plan.used_bullets();
        assert_eq!(used.len(), 2);
        let line = used.iter().find(|b| b.bullet_id == added).unwrap();
        assert_eq!(
            line.rendered_text,
            "Rewrote the ingest path, cutting latency 30%."
        );
        assert_eq!(line.source_text, line.rendered_text, "added verbatim");
        assert!(
            !line.was_reworded,
            "the vault wrote this line, not the model"
        );
    }

    #[test]
    fn an_added_bullet_lands_in_vault_order_not_at_the_end() {
        // The plan holds the second vault bullet; adding the first must put it first.
        let mut s = section("Rajant Health", &[0.9]);
        let vault = vault_for(&s, &[("Second line.", true)]);
        let first = vault[0].roles[0].bullets[0].bullet.id;
        s.roles[0].bullets[0].bullet_id = vault[0].roles[0].bullets[1].bullet.id;
        let mut plan = ResumePlan {
            experience: vec![s],
            ..Default::default()
        };

        plan.apply_edits(
            &vault,
            &[BulletEdit {
                bullet_id: first,
                text: String::new(),
                keep: true,
            }],
        );
        assert_eq!(plan.used_bullets()[0].bullet_id, first);
    }

    #[test]
    fn an_edit_naming_something_outside_the_vault_is_ignored() {
        let s = section("Rajant Health", &[0.9]);
        let vault = vault_for(&s, &[("Inactive line.", false)]);
        let inactive = vault[0].roles[0].bullets[1].bullet.id;
        let mut plan = ResumePlan {
            experience: vec![s],
            ..Default::default()
        };

        plan.apply_edits(
            &vault,
            &[
                BulletEdit {
                    bullet_id: inactive,
                    text: "Inactive line.".into(),
                    keep: true,
                },
                BulletEdit {
                    bullet_id: Uuid::new_v4(),
                    text: "belongs to another vault".into(),
                    keep: true,
                },
            ],
        );
        assert_eq!(plan.bullet_count(), 1, "a hidden bullet stays hidden");
    }

    #[test]
    fn a_project_prints_two_bullets_unless_the_third_stands_with_the_best() {
        let mut lean = ResumePlan {
            projects: vec![section("Car Simulation", &[0.9, 0.8, 0.2])],
            ..Default::default()
        };
        lean.cap_bullets();
        assert_eq!(lean.bullet_count(), 2, "0.2 is nowhere near 0.9");

        let mut strong = ResumePlan {
            projects: vec![section("Car Simulation", &[0.9, 0.85, 0.8])],
            ..Default::default()
        };
        strong.cap_bullets();
        assert_eq!(strong.bullet_count(), 3, "0.8 is 0.89 of 0.9");

        // A job still keeps three, and the base resume's unscored projects start lean.
        let mut work = ResumePlan {
            experience: vec![section("Rajant Health", &[0.9, 0.8, 0.2, 0.1])],
            projects: vec![section("Car Simulation", &[0.0, 0.0, 0.0])],
            ..Default::default()
        };
        work.cap_bullets();
        assert_eq!(work.experience[0].roles[0].bullets.len(), 3);
        assert_eq!(work.projects[0].roles[0].bullets.len(), 2);
    }

    #[test]
    fn what_the_cap_and_the_fit_loop_take_comes_back_in_place() {
        let mut plan = ResumePlan {
            projects: vec![section("Car Simulation", &[0.9, 0.1, 0.8, 0.7])],
            ..Default::default()
        };
        plan.cap_bullets();
        assert_eq!(plan.bullet_count(), 2, "a project starts lean");

        assert!(plan.restore_bullet());
        let texts: Vec<String> = plan
            .used_bullets()
            .into_iter()
            .map(|b| b.rendered_text)
            .collect();
        assert_eq!(texts, ["bullet 0.9", "bullet 0.8", "bullet 0.7"]);
        assert!(
            !plan.restore_bullet(),
            "three is the ceiling — 0.1 stays benched"
        );

        // What the fit loop takes comes back the same way.
        assert!(plan.drop_lowest().is_some());
        assert_eq!(plan.bullet_count(), 2);
        assert!(plan.restore_bullet());
        assert_eq!(plan.bullet_count(), 3);
    }

    /// The grow pass spends room on breadth, not on a fourth line under one experience —
    /// which is the whole point of the cap, and it has to hold after the page is fitted.
    #[test]
    fn the_grow_pass_never_pushes_an_experience_past_the_cap() {
        let mut plan = ResumePlan {
            experience: vec![section("Rajant Health", &[0.9, 0.1, 0.8, 0.7, 0.6])],
            ..Default::default()
        };
        plan.cap_bullets();
        assert_eq!(plan.bullet_count(), 3);
        assert!(!plan.restore_bullet(), "already at the cap");
        assert_eq!(plan.bullet_count(), 3);
    }

    #[test]
    fn a_retired_experience_comes_back_where_it_was_and_stops_being_named() {
        let mut plan = ResumePlan {
            experience: vec![
                scored("Strong Co", 2.4, false),
                scored("Weak Co", 0.2, false),
            ],
            projects: vec![scored("Car Simulation", 1.0, false)],
            ..Default::default()
        };

        assert_eq!(plan.retire_weakest(), Some("Car Simulation".into()));
        assert_eq!(plan.retire_weakest(), Some("Weak Co".into()));
        assert_eq!(plan.retired_names(), ["Car Simulation", "Weak Co"]);

        // Breadth comes back strongest-first, and into the list it left.
        assert!(plan.restore_section());
        assert_eq!(plan.projects[0].org, "Car Simulation");
        assert!(plan.restore_section());
        assert_eq!(plan.experience[1].org, "Weak Co", "back in its old slot");
        assert!(plan.retired_names().is_empty());
        assert!(!plan.restore_section());
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
