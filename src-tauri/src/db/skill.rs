//! The `skills` table and the tag join tables.
//!
//! Matching is always by `slug` or `aliases`, never by `name` — display casing drifts.

use crate::domain::Skill;
use crate::error::Result;
use sqlx::PgPool;
use uuid::Uuid;

/// Case-folded match key for a skill name: lowercase, non-alphanumerics collapsed to `-`.
///
/// "Node.js" -> "node-js", "AWS (EC2)" -> "aws-ec2". Deterministic, so the same typed
/// name always upserts onto the same row.
pub fn slugify(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    let mut pending_sep = false;
    for c in name.chars() {
        if c.is_ascii_alphanumeric() {
            if pending_sep && !out.is_empty() {
                out.push('-');
            }
            pending_sep = false;
            out.extend(c.to_lowercase());
        } else {
            pending_sep = true;
        }
    }
    out
}

pub async fn list(pool: &PgPool) -> Result<Vec<Skill>> {
    Ok(sqlx::query_as::<_, Skill>(
        "SELECT id, name, slug, category, aliases FROM skills ORDER BY name",
    )
    .fetch_all(pool)
    .await?)
}

/// Returns the skill for `name`, creating it if the slug is new.
///
/// An existing row keeps its display name and aliases; a first-seen name defines them.
pub async fn upsert_by_name(pool: &PgPool, name: &str) -> Result<Skill> {
    let slug = slugify(name);
    Ok(sqlx::query_as::<_, Skill>(
        "INSERT INTO skills (name, slug) VALUES ($1, $2)
         ON CONFLICT (slug) DO UPDATE SET slug = EXCLUDED.slug
         RETURNING id, name, slug, category, aliases",
    )
    .bind(name.trim())
    .bind(&slug)
    .fetch_one(pool)
    .await?)
}

pub async fn set_bullet_skills(pool: &PgPool, bullet_id: Uuid, names: &[String]) -> Result<()> {
    let mut ids = Vec::with_capacity(names.len());
    for n in names.iter().filter(|n| !n.trim().is_empty()) {
        ids.push(upsert_by_name(pool, n).await?.id);
    }
    sqlx::query("DELETE FROM bullet_skills WHERE bullet_id = $1")
        .bind(bullet_id)
        .execute(pool)
        .await?;
    sqlx::query(
        "INSERT INTO bullet_skills (bullet_id, skill_id)
         SELECT $1, unnest($2::uuid[]) ON CONFLICT DO NOTHING",
    )
    .bind(bullet_id)
    .bind(&ids)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn set_experience_skills(
    pool: &PgPool,
    experience_id: Uuid,
    names: &[String],
) -> Result<()> {
    let mut ids = Vec::with_capacity(names.len());
    for n in names.iter().filter(|n| !n.trim().is_empty()) {
        ids.push(upsert_by_name(pool, n).await?.id);
    }
    sqlx::query("DELETE FROM experience_skills WHERE experience_id = $1")
        .bind(experience_id)
        .execute(pool)
        .await?;
    sqlx::query(
        "INSERT INTO experience_skills (experience_id, skill_id)
         SELECT $1, unnest($2::uuid[]) ON CONFLICT DO NOTHING",
    )
    .bind(experience_id)
    .bind(&ids)
    .execute(pool)
    .await?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::slugify;

    #[test]
    fn slugs_are_case_folded_and_punctuation_free() {
        assert_eq!(slugify("PostgreSQL"), "postgresql");
        assert_eq!(slugify("Node.js"), "node-js");
        assert_eq!(slugify("AWS (EC2, Lambda)"), "aws-ec2-lambda");
        assert_eq!(slugify("  C  "), "c");
        assert_eq!(slugify("HTML/CSS"), "html-css");
    }
}
