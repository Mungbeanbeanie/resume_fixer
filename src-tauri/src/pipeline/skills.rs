//! Resolving free text to a skill row.
//!
//! Matching is by `slug` plus `aliases`, always case-folded. `skills.name` is a display
//! string and is never matched on.

use crate::db::skill::slugify;
use crate::domain::Skill;
use std::collections::HashMap;

pub struct SkillIndex {
    /// slug-or-alias, case-folded -> canonical slug
    by_key: HashMap<String, String>,
    /// canonical slug -> display name
    names: HashMap<String, String>,
    /// display names of the skills marked to print, in `skills` table order
    always: Vec<String>,
}

impl SkillIndex {
    pub fn new(skills: &[Skill]) -> Self {
        let mut by_key = HashMap::new();
        let mut names = HashMap::new();
        let mut always = Vec::new();
        for s in skills {
            by_key.insert(s.slug.clone(), s.slug.clone());
            by_key.insert(slugify(&s.name), s.slug.clone());
            for a in &s.aliases {
                by_key.insert(slugify(a), s.slug.clone());
            }
            names.insert(s.slug.clone(), s.name.clone());
            if s.always_list {
                always.push(s.name.clone());
            }
        }
        SkillIndex {
            by_key,
            names,
            always,
        }
    }

    /// The canonical slug for a typed or model-supplied name, if the vault knows it.
    pub fn resolve(&self, text: &str) -> Option<&str> {
        self.by_key.get(&slugify(text)).map(String::as_str)
    }

    /// The user's own spelling of a skill, for printing on the resume.
    pub fn display(&self, slug: &str) -> Option<&str> {
        self.names.get(slug).map(String::as_str)
    }

    /// The skills line for a tailored resume: what the posting asked for and the vault
    /// knows, in the posting's order, then everything the user marked to always print.
    ///
    /// The posting's priorities lead because a recruiter reads left to right, but a skill
    /// the user checked prints whether or not this posting mentioned it.
    pub fn line<'a>(&self, wanted: impl IntoIterator<Item = &'a str>) -> Vec<String> {
        let mut out: Vec<String> = self
            .resolve_all(wanted)
            .iter()
            .filter_map(|slug| self.display(slug).map(str::to_string))
            .collect();
        for name in &self.always {
            if !out.contains(name) {
                out.push(name.clone());
            }
        }
        out
    }

    /// Resolves a list of names to canonical slugs, dropping what the vault does not have.
    pub fn resolve_all<'a>(&self, names: impl IntoIterator<Item = &'a str>) -> Vec<String> {
        let mut out: Vec<String> = Vec::new();
        for n in names {
            if let Some(slug) = self.resolve(n) {
                if !out.iter().any(|s| s == slug) {
                    out.push(slug.to_string());
                }
            }
        }
        out
    }
}

#[cfg(test)]
pub fn test_skills(pairs: &[(&str, &[&str])]) -> Vec<Skill> {
    pairs
        .iter()
        .map(|(name, aliases)| Skill {
            id: uuid::Uuid::new_v4(),
            name: name.to_string(),
            slug: slugify(name),
            category: None,
            aliases: aliases.iter().map(|a| a.to_string()).collect(),
            always_list: false,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aliases_and_casing_resolve_to_one_slug() {
        let skills = test_skills(&[("PostgreSQL", &["postgres", "psql", "pg"])]);
        let idx = SkillIndex::new(&skills);
        assert_eq!(idx.resolve("Postgres"), Some("postgresql"));
        assert_eq!(idx.resolve("POSTGRESQL"), Some("postgresql"));
        assert_eq!(idx.resolve("pg"), Some("postgresql"));
        assert_eq!(idx.resolve("MySQL"), None);
        assert_eq!(idx.display("postgresql"), Some("PostgreSQL"));
    }

    #[test]
    fn the_line_leads_with_the_posting_then_adds_what_is_always_listed() {
        let mut all = test_skills(&[("Python", &[]), ("Rust", &[]), ("SQL", &["postgres"])]);
        all[1].always_list = true;
        all[2].always_list = true;
        let idx = SkillIndex::new(&all);
        // SQL is both asked for and always listed: it leads, and does not repeat.
        assert_eq!(
            idx.line(["Postgres", "Python", "Go"]),
            ["SQL", "Python", "Rust"]
        );
    }

    #[test]
    fn unknown_names_are_dropped_not_invented() {
        let skills = test_skills(&[("Python", &[]), ("Java", &[])]);
        let idx = SkillIndex::new(&skills);
        assert_eq!(
            idx.resolve_all(["Python", "Rust", "java"]),
            ["python", "java"]
        );
    }
}
