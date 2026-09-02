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

/// Every skill, the checked ones first in the order they were checked.
///
/// This order is what the base resume prints, so it is the user's own: `listed_at` is
/// stamped when a box is ticked. Everything unchecked sorts alphabetically behind it.
pub async fn list(pool: &PgPool) -> Result<Vec<Skill>> {
    Ok(sqlx::query_as::<_, Skill>(
        "SELECT id, name, slug, category, aliases, always_list FROM skills
         ORDER BY listed_at NULLS LAST, name",
    )
    .fetch_all(pool)
    .await?)
}

/// Returns the skill for `name`, creating it if the slug is new.
///
/// An existing row keeps its display name, aliases and listing flag; a first-seen name
/// defines them and starts listed, since a skill worth tagging is worth printing. The flag
/// is never rewritten here: re-saving a bullet must not undo a box the user unchecked, and
/// it must not move a checked one to the end of the printed line either.
pub async fn upsert_by_name(pool: &PgPool, name: &str) -> Result<Skill> {
    let slug = slugify(name);
    Ok(sqlx::query_as::<_, Skill>(
        "INSERT INTO skills (name, slug, always_list, listed_at) VALUES ($1, $2, TRUE, now())
         ON CONFLICT (slug) DO UPDATE SET slug = EXCLUDED.slug
         RETURNING id, name, slug, category, aliases, always_list",
    )
    .bind(name.trim())
    .bind(&slug)
    .fetch_one(pool)
    .await?)
}

/// Returns the skill for `name`, creating it if the slug is new, and marks it to print on
/// the base resume.
///
/// An existing row keeps its display name and aliases, as `upsert_by_name` does — only the
/// listing flag is written, and a row already listed keeps the place it prints in.
pub async fn add_listed(pool: &PgPool, name: &str) -> Result<Skill> {
    let slug = slugify(name);
    Ok(sqlx::query_as::<_, Skill>(
        "INSERT INTO skills (name, slug, always_list, listed_at) VALUES ($1, $2, TRUE, now())
         ON CONFLICT (slug) DO UPDATE
           SET always_list = TRUE, listed_at = COALESCE(skills.listed_at, now())
         RETURNING id, name, slug, category, aliases, always_list",
    )
    .bind(name.trim())
    .bind(&slug)
    .fetch_one(pool)
    .await?)
}

/// Turns the base-resume listing for one skill on or off.
///
/// Never deletes the row: the slug and aliases still serve job-posting matching, and the
/// tags pointing at it would cascade away with it.
///
/// Checking a skill stamps `listed_at`, which is the order the base resume prints in.
/// Unchecking clears it, so ticking the box again puts the skill at the end of the line —
/// the only way to move one, and the reason turning it on is idempotent.
pub async fn set_always_list(pool: &PgPool, id: Uuid, on: bool) -> Result<()> {
    sqlx::query(
        "UPDATE skills
            SET always_list = $2,
                listed_at = CASE WHEN $2 THEN COALESCE(listed_at, now()) ELSE NULL END
          WHERE id = $1",
    )
    .bind(id)
    .bind(on)
    .execute(pool)
    .await?;
    Ok(())
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
