//! How good an experience is on its own, with no posting to match against.
//!
//! `retrieval` scores a bullet's *fit* for one job. This scores an experience's *merit* —
//! what the base resume ranks by, and the tiebreak when a tailored resume has to retire a
//! whole entry to reach one page.

use crate::domain::ExperienceDetail;
use crate::pipeline::grounding;
use crate::pipeline::retrieval::recency_factor;
use chrono::{NaiveDate, Utc};

/// A quantified outcome is the single most persuasive thing on a resume: recruiter surveys
/// put it first by a wide margin, and the standing advice is that something like half of a
/// resume's bullets should carry a number. It outweighs everything else here.
///
/// Recency is real but secondary — the "10 to 15 year rule" exists, yet every source that
/// ranks the two puts relevance and evidence above age, and a student's whole history fits
/// inside the four-year horizon `retrieval` already uses. Skill density is the weakest
/// term: it stands in for substance when nothing else separates two entries.
///
/// These sources are career-advice publishers repeating second-hand figures, not primary
/// research. The *ordering* of the three terms is well supported and should not be
/// reshuffled without a reason. The exact ratios are a judgement call — tune them against
/// real output, and do not read more precision into them than that.
const W_QUANTIFIED: f32 = 2.0;
const W_RECENCY: f32 = 1.0;
const W_SKILL_DENSITY: f32 = 0.5;

/// Bullets past this many add nothing to the score — an entry is judged on its best work,
/// not on how much of it was written down.
const SCORED_BULLETS: usize = 3;

/// True when a bullet states an outcome you can check: a count, a percentage, a duration.
fn is_quantified(text: &str) -> bool {
    !grounding::numbers(text).is_empty()
}

/// The merit of one experience, independent of any posting.
///
/// Zero for an entry with no active bullets: there is nothing to judge, and nothing that
/// would print. Education and pinned entries are exempt from retirement elsewhere, so a
/// low score here never removes them.
pub fn score(detail: &ExperienceDetail, today: NaiveDate) -> f32 {
    let bullets: Vec<&crate::domain::BulletDetail> = detail
        .roles
        .iter()
        .filter(|r| r.role.is_active)
        .flat_map(|r| r.bullets.iter())
        .filter(|b| b.bullet.is_active)
        .collect();
    if bullets.is_empty() {
        return 0.0;
    }

    let judged = bullets.len().min(SCORED_BULLETS) as f32;
    let quantified = bullets
        .iter()
        .take(SCORED_BULLETS)
        .filter(|b| is_quantified(&b.bullet.text))
        .count() as f32
        / judged;

    let tags = bullets
        .iter()
        .take(SCORED_BULLETS)
        .map(|b| b.skills.len())
        .sum::<usize>() as f32
        / judged;
    // Three distinct skills on a bullet is already dense; more says little extra.
    let density = (tags / 3.0).min(1.0);

    // The experience is as recent as its most recent role.
    let recency = detail
        .roles
        .iter()
        .filter(|r| r.role.is_active)
        .map(|r| recency_factor(r.role.end_date, today))
        .fold(0.0_f32, f32::max);

    W_QUANTIFIED * quantified + W_RECENCY * recency + W_SKILL_DENSITY * density
}

/// `score` for every experience, keyed by id, against today's date.
pub fn score_all(vault: &[ExperienceDetail]) -> std::collections::HashMap<uuid::Uuid, f32> {
    let today = Utc::now().date_naive();
    vault
        .iter()
        .map(|d| (d.experience.id, score(d, today)))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::*;
    use uuid::Uuid;

    fn detail(texts: &[&str], tags_each: usize, years_ago: i64) -> ExperienceDetail {
        let role_id = Uuid::new_v4();
        ExperienceDetail {
            experience: Experience {
                id: Uuid::new_v4(),
                kind: ExperienceKind::Work,
                org_name: "Rajant Health".into(),
                location: None,
                url: None,
                tech_line: None,
                display_order: 0,
                is_active: true,
                is_pinned: false,
            },
            skills: vec![],
            roles: vec![RoleDetail {
                role: Role {
                    id: role_id,
                    experience_id: Uuid::new_v4(),
                    title: "Intern".into(),
                    location: None,
                    start_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
                    end_date: Some(
                        Utc::now().date_naive() - chrono::Duration::days(365 * years_ago),
                    ),
                    date_override: None,
                    display_order: 0,
                    is_active: true,
                },
                bullets: texts
                    .iter()
                    .map(|t| BulletDetail {
                        bullet: Bullet {
                            id: Uuid::new_v4(),
                            role_id,
                            text: (*t).into(),
                            display_order: 0,
                            is_active: true,
                        },
                        skills: (0..tags_each)
                            .map(|i| Skill {
                                id: Uuid::new_v4(),
                                name: format!("Skill{i}"),
                                slug: format!("skill{i}"),
                                category: None,
                                aliases: vec![],
                                always_list: false,
                            })
                            .collect(),
                        variants: vec![],
                    })
                    .collect(),
            }],
        }
    }

    #[test]
    fn a_quantified_experience_outranks_a_more_recent_vague_one() {
        let today = Utc::now().date_naive();
        let measured = detail(&["Cut request latency by 25%."], 1, 3);
        let vague = detail(&["Worked on backend services."], 1, 0);
        assert!(
            score(&measured, today) > score(&vague, today),
            "evidence beats freshness"
        );
    }

    #[test]
    fn recency_separates_two_otherwise_equal_entries() {
        let today = Utc::now().date_naive();
        let now = detail(&["Cut latency by 25%."], 1, 0);
        let old = detail(&["Cut latency by 25%."], 1, 4);
        assert!(score(&now, today) > score(&old, today));
    }

    #[test]
    fn an_experience_with_nothing_to_print_scores_zero() {
        let today = Utc::now().date_naive();
        assert_eq!(score(&detail(&[], 0, 0), today), 0.0);
    }
}
