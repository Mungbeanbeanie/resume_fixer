//! The gate every reworded bullet passes before it can be rendered.
//!
//! A rewrite is allowed to reorder, compress, or re-emphasize its source. It is not
//! allowed to add anything. These checks are deterministic and run on every rewrite,
//! including ones from a model that has behaved perfectly so far.
//!
//! A failure is never fatal: the caller falls back to the verbatim source bullet.

use super::skills::SkillIndex;
use std::fmt;

/// Lower bound: below 0.7x the source, a "compression" has dropped a fact rather than
/// tightened wording. Upper bound: above 1.25x, something was added — the strong-bullet
/// pattern surfaces facts, it does not lengthen them by a quarter.
const MIN_RATIO: f32 = 0.7;
const MAX_RATIO: f32 = 1.25;

/// Claims a resume cannot make about itself. Allowed only when the source already says it.
const SUPERLATIVES: [&str; 6] = [
    "first ever",
    "industry-leading",
    "award-winning",
    "best-in-class",
    "single-handedly",
    "revolutionary",
];

/// The model was told plain text. Any of these is a sign it ignored the instruction, and
/// sanitizing it through would mean trusting the rest of that answer.
const MARKUP: [char; 8] = ['\\', '{', '}', '$', '^', '_', '~', '#'];

#[derive(Debug, Clone, PartialEq)]
pub enum Rejection {
    NewNumber(String),
    NewEntity(String),
    Length { ratio: f32 },
    Superlative(String),
    Markup(char),
    Empty,
}

impl fmt::Display for Rejection {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Rejection::NewNumber(n) => write!(f, "introduces a number not in the original: {n}"),
            Rejection::NewEntity(e) => write!(f, "introduces something not in the original: {e}"),
            Rejection::Length { ratio } => {
                write!(f, "length is {:.0}% of the original", ratio * 100.0)
            }
            Rejection::Superlative(s) => write!(f, "adds the claim \"{s}\""),
            Rejection::Markup(c) => write!(f, "contains markup: {c}"),
            Rejection::Empty => write!(f, "is empty"),
        }
    }
}

/// Numeric tokens, normalized: thousands separators removed, trailing punctuation
/// dropped, a `%` or `+` kept because it changes the claim, an attached unit kept
/// because `15ms` and `15s` are different facts.
fn numbers(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if !chars[i].is_ascii_digit() {
            i += 1;
            continue;
        }
        let mut tok = String::new();
        while i < chars.len()
            && (chars[i].is_ascii_digit()
                || chars[i] == ','
                || (chars[i] == '.' && chars.get(i + 1).is_some_and(char::is_ascii_digit)))
        {
            if chars[i] != ',' {
                tok.push(chars[i]);
            }
            i += 1;
        }
        // A unit or percent glued to the digits is part of the claim.
        while i < chars.len()
            && (chars[i].is_ascii_alphabetic() || chars[i] == '%' || chars[i] == '+')
        {
            tok.extend(chars[i].to_lowercase());
            i += 1;
        }
        out.push(tok);
    }
    out
}

fn words(text: &str) -> Vec<&str> {
    text.split(|c: char| c.is_whitespace())
        .map(|w| w.trim_matches(|c: char| !c.is_alphanumeric() && c != '-' && c != '.' && c != '+'))
        .filter(|w| !w.is_empty())
        .collect()
}

fn contains_fold(haystack: &str, needle: &str) -> bool {
    haystack.to_lowercase().contains(&needle.to_lowercase())
}

/// Verifies a rewrite against its source.
///
/// `bullet_slugs` are the skills tagged on the source bullet — a rewrite may name a
/// technology the tags record even when the prose does not spell it out. `index` is the
/// vault's whole skill list, used to notice a technology the rewrite introduced.
pub fn check(
    source: &str,
    rewrite: &str,
    bullet_slugs: &[String],
    index: &SkillIndex,
) -> Result<(), Rejection> {
    let rewrite = rewrite.trim();
    if rewrite.is_empty() {
        return Err(Rejection::Empty);
    }
    if let Some(c) = rewrite.chars().find(|c| MARKUP.contains(c)) {
        return Err(Rejection::Markup(c));
    }

    let ratio = rewrite.chars().count() as f32 / source.chars().count().max(1) as f32;
    if !(MIN_RATIO..=MAX_RATIO).contains(&ratio) {
        return Err(Rejection::Length { ratio });
    }

    let source_numbers = numbers(source);
    for n in numbers(rewrite) {
        if !source_numbers.contains(&n) {
            return Err(Rejection::NewNumber(n));
        }
    }

    let lower_source = source.to_lowercase();
    for s in SUPERLATIVES {
        if rewrite.to_lowercase().contains(s) && !lower_source.contains(s) {
            return Err(Rejection::Superlative(s.to_string()));
        }
    }

    let source_words = words(source);
    for (i, word) in words(rewrite).into_iter().enumerate() {
        // The opening word is capitalized by grammar, not because it names anything —
        // a rewrite is allowed to open with a different verb.
        let opening = i == 0;
        if grounded(word, opening, source, &source_words, bullet_slugs, index) {
            continue;
        }
        // "Java-based" is grounded when "Java" is: check the parts of a compound.
        if word.contains('-')
            && word.split('-').all(|p| {
                p.is_empty() || grounded(p, false, source, &source_words, bullet_slugs, index)
            })
        {
            continue;
        }
        return Err(Rejection::NewEntity(word.to_string()));
    }
    Ok(())
}

/// True when a token carries no claim the source does not already make.
fn grounded(
    word: &str,
    opening: bool,
    source: &str,
    source_words: &[&str],
    bullet_slugs: &[String],
    index: &SkillIndex,
) -> bool {
    let known_skill = index.resolve(word);
    let names_something =
        word.chars().next().is_some_and(char::is_uppercase) && word.chars().count() > 1;
    if known_skill.is_none() && (opening || !names_something) {
        return true;
    }
    if source_words.iter().any(|w| w.eq_ignore_ascii_case(word)) || contains_fold(source, word) {
        return true;
    }
    match known_skill {
        // The source may spell a tagged skill differently than the rewrite does.
        Some(slug) => {
            bullet_slugs.iter().any(|s| s == slug)
                || source_words.iter().any(|w| index.resolve(w) == Some(slug))
        }
        None => false,
    }
}

#[cfg(test)]
mod tests {
    use super::super::skills::{test_skills, SkillIndex};
    use super::*;

    fn index() -> SkillIndex {
        SkillIndex::new(&test_skills(&[
            ("Python", &[]),
            ("Java", &[]),
            ("PostgreSQL", &["postgres"]),
            ("Rust", &[]),
            ("Docker", &[]),
            ("Kubernetes", &["k8s"]),
            ("AWS", &[]),
        ]))
    }

    /// `(source, rewrite, should_pass, why)` — the fixture the plan calls for.
    const CASES: &[(&str, &str, bool, &str)] = &[
        (
            "Worked on backend services in Java for banking applications, improving request latency by 25%.",
            "Designed Java-based services for banking applications, improving request latency by 25%.",
            true,
            "legitimate: reorders and tightens, adds nothing",
        ),
        (
            "Implemented lock-free UART data pipeline using ring buffers sustaining 460 kbps continuous transfer.",
            "Built lock-free UART pipeline with ring buffers sustaining 460 kbps continuous transfer.",
            true,
            "legitimate: opening verb may change",
        ),
        (
            "Improved request latency by 25%.",
            "Improved request latency by 40%.",
            false,
            "swapped number",
        ),
        (
            "Improved request latency by 25%.",
            "Improved request latency by 25% across 12 services.",
            false,
            "invented a second number",
        ),
        (
            "Built a serverless function to automate campaign reporting.",
            "Built a serverless Python function to automate campaign reporting.",
            false,
            "added a technology the bullet is not tagged with",
        ),
        (
            "Built a data pipeline for satellite launch datasets.",
            "Built a data pipeline for NASA satellite launch datasets.",
            false,
            "invented an employer or partner",
        ),
        (
            "Deployed a REST API on AWS EC2 with Docker and Nginx.",
            "Deployed a REST API on AWS EC2 with Docker, Nginx, and Kubernetes.",
            false,
            "inflated the stack",
        ),
        (
            "Delivered a proof-of-concept validating a next-generation hardware platform.",
            "Delivered the industry-leading proof-of-concept validating a hardware platform.",
            false,
            "unearned superlative",
        ),
        (
            "Designed semantic embedding based algorithms for deduplication and provenance.",
            "Designed algorithms.",
            false,
            "compressed past the point of keeping the facts",
        ),
        (
            "Wrote a parser.",
            "Wrote a robust, production-grade, fully tested recursive descent parser handling every edge case.",
            false,
            "expansion",
        ),
        (
            "Automated campaign data extraction with a Lambda function.",
            "Automated campaign data extraction with a \\textbf{Lambda} function.",
            false,
            "markup in the answer",
        ),
    ];

    #[test]
    fn every_fabrication_is_caught_and_every_honest_rewrite_survives() {
        let idx = index();
        for (source, rewrite, should_pass, why) in CASES {
            let got = check(source, rewrite, &[], &idx);
            assert_eq!(
                got.is_ok(),
                *should_pass,
                "{why}: {source} -> {rewrite} gave {got:?}"
            );
        }
    }

    #[test]
    fn a_tagged_skill_may_be_named_even_when_the_prose_omits_it() {
        let idx = index();
        let source = "Built a serverless function to automate campaign reporting daily.";
        let rewrite = "Built a serverless Python function to automate campaign reporting.";
        assert!(check(source, rewrite, &[], &idx).is_err());
        assert!(check(source, rewrite, &["python".into()], &idx).is_ok());
    }

    #[test]
    fn an_alias_of_a_skill_already_present_is_not_a_new_entity() {
        let idx = index();
        let source = "Modeled 10 tables in Postgres with row-level security enabled.";
        let rewrite = "Modeled 10 PostgreSQL tables with row-level security enabled.";
        assert!(check(source, rewrite, &[], &idx).is_ok());
    }

    #[test]
    fn numbers_normalize_separators_but_not_units() {
        assert_eq!(
            numbers("2,000 users and 460 kbps at 15ms"),
            ["2000", "460", "15ms"]
        );
        assert_eq!(
            numbers("20+ endpoints, 3--5% of payloads"),
            ["20+", "3", "5%"]
        );
        let idx = index();
        assert!(check("Served 2,000 users.", "Served 2000 users.", &[], &idx).is_ok());
        assert!(check(
            "Held 15ms frame latency down.",
            "Held 15s frame latency down.",
            &[],
            &idx
        )
        .is_err());
    }

    #[test]
    fn length_bounds_are_inclusive_at_the_edges() {
        let idx = index();
        let source = "a".repeat(100);
        assert!(check(&source, &"b".repeat(70), &[], &idx).is_ok());
        assert!(check(&source, &"b".repeat(69), &[], &idx).is_err());
        assert!(check(&source, &"b".repeat(125), &[], &idx).is_ok());
        assert!(check(&source, &"b".repeat(126), &[], &idx).is_err());
    }
}
