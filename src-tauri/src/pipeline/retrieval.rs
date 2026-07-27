//! Deterministic candidate scoring. No model runs here.

use super::skills::SkillIndex;
use crate::domain::Candidate;
use chrono::{Datelike, NaiveDate, Utc};
use std::collections::HashSet;

/// Bullet tags are the precise signal, so they outweigh experience tags 2:1. Experience
/// tags still count: they catch relevance the bullet prose never spells out. Literal token
/// overlap is a weak tiebreak for skills nobody has tagged yet, and recency is weaker
/// still — it orders otherwise-equal bullets, it does not outrank a match.
const W_BULLET_SKILL: f32 = 2.0;
const W_EXPERIENCE_SKILL: f32 = 1.0;
const W_LITERAL: f32 = 0.5;
const W_RECENCY: f32 = 0.3;

/// A role four years gone is as relevant as no role at all for a student's resume.
const RECENCY_HORIZON_YEARS: f32 = 4.0;

/// The model reads a shortlist, not the whole vault: enough for it to compose a page
/// with room to drop, small enough to stay inside a 35B model's useful attention.
pub const SHORTLIST: usize = 24;

fn recency_factor(end_date: Option<NaiveDate>, today: NaiveDate) -> f32 {
    let Some(end) = end_date else { return 1.0 }; // still there
    let years = (today.num_days_from_ce() - end.num_days_from_ce()) as f32 / 365.25;
    (1.0 - years / RECENCY_HORIZON_YEARS).clamp(0.0, 1.0)
}

/// Scores every candidate against the posting's hard skills.
///
/// `wanted` are raw strings from the parsed posting; they are resolved to slugs here so a
/// posting saying "Postgres" matches a bullet tagged "postgresql".
pub fn score(candidates: &mut [Candidate], wanted: &[String], index: &SkillIndex) {
    let today = Utc::now().date_naive();
    let slugs: HashSet<String> = index
        .resolve_all(wanted.iter().map(String::as_str))
        .into_iter()
        .collect();
    let literals: Vec<String> = wanted.iter().map(|w| w.to_lowercase()).collect();

    for c in candidates.iter_mut() {
        let bullet: f32 = c
            .skills
            .iter()
            .filter(|(slug, _)| slugs.contains(slug))
            .map(|(_, weight)| *weight)
            .sum();
        let experience = c
            .experience_skills
            .iter()
            .filter(|slug| slugs.contains(*slug))
            .count() as f32;
        let lower = c.text.to_lowercase();
        let literal = literals.iter().filter(|w| lower.contains(*w)).count() as f32;

        c.score = W_BULLET_SKILL * bullet
            + W_EXPERIENCE_SKILL * experience
            + W_LITERAL * literal
            + W_RECENCY * recency_factor(c.end_date, today);
    }
}

/// The shortlist handed to the model: the highest scorers, plus a floor of one bullet per
/// experience so a whole job never silently disappears from consideration.
pub fn shortlist(
    mut candidates: Vec<Candidate>,
    wanted: &[String],
    index: &SkillIndex,
) -> Vec<Candidate> {
    score(&mut candidates, wanted, index);
    candidates.sort_by(|a, b| b.score.total_cmp(&a.score));

    let mut kept: Vec<Candidate> = Vec::with_capacity(SHORTLIST);
    let mut represented: HashSet<uuid::Uuid> = HashSet::new();
    for c in &candidates {
        if kept.len() >= SHORTLIST {
            break;
        }
        represented.insert(c.experience_id);
        kept.push(c.clone());
    }
    for c in &candidates {
        if !represented.contains(&c.experience_id) {
            represented.insert(c.experience_id);
            kept.push(c.clone());
        }
    }
    kept
}

#[cfg(test)]
mod tests {
    use super::super::skills::{test_skills, SkillIndex};
    use super::*;
    use uuid::Uuid;

    fn candidate(text: &str, skills: &[(&str, f32)], exp: Uuid, years_ago: i64) -> Candidate {
        Candidate {
            bullet_id: Uuid::new_v4(),
            text: text.into(),
            role_id: Uuid::new_v4(),
            role_title: "Intern".into(),
            end_date: Some(Utc::now().date_naive() - chrono::Duration::days(365 * years_ago)),
            experience_id: exp,
            org_name: "Org".into(),
            skills: skills.iter().map(|(s, w)| (s.to_string(), *w)).collect(),
            experience_skills: vec![],
            score: 0.0,
        }
    }

    #[test]
    fn a_tagged_bullet_outranks_a_merely_recent_one() {
        let idx = SkillIndex::new(&test_skills(&[("Python", &[]), ("Java", &[])]));
        let e = Uuid::new_v4();
        let mut cands = vec![
            candidate("Wrote documentation", &[], e, 0),
            candidate("Built a service", &[("python", 1.0)], e, 3),
        ];
        score(&mut cands, &["Python".into()], &idx);
        assert!(cands[1].score > cands[0].score);
    }

    #[test]
    fn a_posting_alias_matches_the_canonical_tag() {
        let idx = SkillIndex::new(&test_skills(&[("PostgreSQL", &["postgres"])]));
        let e = Uuid::new_v4();
        let mut cands = vec![candidate("Modeled tables", &[("postgresql", 1.0)], e, 0)];
        score(&mut cands, &["Postgres".into()], &idx);
        assert!(cands[0].score >= 2.0);
    }

    #[test]
    fn recency_decays_to_nothing_at_the_horizon() {
        let today = NaiveDate::from_ymd_opt(2026, 7, 26).unwrap();
        assert_eq!(recency_factor(None, today), 1.0);
        assert!(recency_factor(Some(today), today) > 0.99);
        assert_eq!(
            recency_factor(NaiveDate::from_ymd_opt(2020, 1, 1), today),
            0.0
        );
        let two_years = NaiveDate::from_ymd_opt(2024, 7, 26).unwrap();
        assert!((recency_factor(Some(two_years), today) - 0.5).abs() < 0.01);
    }

    #[test]
    fn every_experience_keeps_a_seat_even_past_the_shortlist() {
        let idx = SkillIndex::new(&test_skills(&[("Python", &[])]));
        let hot = Uuid::new_v4();
        let forgotten = Uuid::new_v4();
        let mut cands: Vec<Candidate> = (0..SHORTLIST + 5)
            .map(|_| candidate("Built a service", &[("python", 1.0)], hot, 0))
            .collect();
        cands.push(candidate("Filed paperwork", &[], forgotten, 3));

        let kept = shortlist(cands, &["Python".into()], &idx);
        assert_eq!(kept.len(), SHORTLIST + 1);
        assert!(kept.iter().any(|c| c.experience_id == forgotten));
    }
}
