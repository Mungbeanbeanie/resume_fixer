//! `experiences` plus the read that assembles the whole Vault tree.

use crate::domain::*;
use crate::error::{AppError, Result};
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

const COLS: &str = "id, kind, org_name, location, url, tech_line, display_order, is_active";

pub async fn list(pool: &PgPool) -> Result<Vec<Experience>> {
    Ok(sqlx::query_as::<_, Experience>(&format!(
        "SELECT {COLS} FROM experiences ORDER BY display_order, created_at"
    ))
    .fetch_all(pool)
    .await?)
}

pub async fn upsert(pool: &PgPool, input: &ExperienceInput) -> Result<Experience> {
    Ok(sqlx::query_as::<_, Experience>(&format!(
        "INSERT INTO experiences (id, kind, org_name, location, url, tech_line, display_order, is_active)
         VALUES (COALESCE($1, gen_random_uuid()), $2, $3, $4, $5, $6, $7, $8)
         ON CONFLICT (id) DO UPDATE
           SET kind = EXCLUDED.kind, org_name = EXCLUDED.org_name, location = EXCLUDED.location,
               url = EXCLUDED.url, tech_line = EXCLUDED.tech_line,
               display_order = EXCLUDED.display_order, is_active = EXCLUDED.is_active,
               updated_at = now()
         RETURNING {COLS}"
    ))
    .bind(input.id)
    .bind(input.kind)
    .bind(&input.org_name)
    .bind(&input.location)
    .bind(&input.url)
    .bind(&input.tech_line)
    .bind(input.display_order)
    .bind(input.is_active)
    .fetch_one(pool)
    .await?)
}

/// Deletes the experience and, by cascade, its roles and bullets.
///
/// Rejects the delete when a bullet underneath it is cited by a rendered resume —
/// `resume_bullets` holds `ON DELETE RESTRICT` so provenance cannot be orphaned.
pub async fn delete(pool: &PgPool, id: Uuid) -> Result<()> {
    let n = sqlx::query("DELETE FROM experiences WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    if n == 0 {
        return Err(AppError::NotFound("experience".into()));
    }
    Ok(())
}

#[derive(sqlx::FromRow)]
struct SkillTag {
    owner: Uuid,
    id: Uuid,
    name: String,
    slug: String,
    category: Option<String>,
    aliases: Vec<String>,
    always_list: bool,
}

impl SkillTag {
    fn split(self) -> (Uuid, Skill) {
        (
            self.owner,
            Skill {
                id: self.id,
                name: self.name,
                slug: self.slug,
                category: self.category,
                aliases: self.aliases,
                always_list: self.always_list,
            },
        )
    }
}

fn group<T>(rows: Vec<(Uuid, T)>) -> HashMap<Uuid, Vec<T>> {
    let mut map: HashMap<Uuid, Vec<T>> = HashMap::new();
    for (k, v) in rows {
        map.entry(k).or_default().push(v);
    }
    map
}

/// The whole Vault in one call: experiences → roles → bullets, each with its skills.
///
/// Six flat queries stitched in Rust rather than a nested join, so no row is duplicated
/// across the wire. Roles come back newest first — the most recent title owns the org header.
pub async fn list_details(pool: &PgPool) -> Result<Vec<ExperienceDetail>> {
    let experiences = list(pool).await?;

    let roles = sqlx::query_as::<_, Role>(
        "SELECT id, experience_id, title, location, start_date, end_date, date_override,
                display_order, is_active
         FROM roles ORDER BY start_date DESC, display_order",
    )
    .fetch_all(pool)
    .await?;

    let bullets = sqlx::query_as::<_, Bullet>(
        "SELECT id, role_id, text, display_order, is_active FROM bullets
         ORDER BY display_order, created_at",
    )
    .fetch_all(pool)
    .await?;

    let bullet_skills = sqlx::query_as::<_, SkillTag>(
        "SELECT bs.bullet_id AS owner, s.id, s.name, s.slug, s.category, s.aliases, s.always_list
         FROM bullet_skills bs JOIN skills s ON s.id = bs.skill_id ORDER BY s.name",
    )
    .fetch_all(pool)
    .await?;

    let experience_skills = sqlx::query_as::<_, SkillTag>(
        "SELECT es.experience_id AS owner, s.id, s.name, s.slug, s.category, s.aliases, s.always_list
         FROM experience_skills es JOIN skills s ON s.id = es.skill_id ORDER BY s.name",
    )
    .fetch_all(pool)
    .await?;

    let variants = sqlx::query_as::<_, BulletVariant>(
        "SELECT id, bullet_id, text, origin, approved_at FROM bullet_variants
         ORDER BY created_at DESC",
    )
    .fetch_all(pool)
    .await?;

    let mut bskills = group(bullet_skills.into_iter().map(SkillTag::split).collect());
    let mut eskills = group(experience_skills.into_iter().map(SkillTag::split).collect());
    let mut bvariants = group(variants.into_iter().map(|v| (v.bullet_id, v)).collect());
    let mut by_role = group(bullets.into_iter().map(|b| (b.role_id, b)).collect());
    let mut by_experience = group(roles.into_iter().map(|r| (r.experience_id, r)).collect());

    Ok(experiences
        .into_iter()
        .map(|experience| {
            let roles = by_experience
                .remove(&experience.id)
                .unwrap_or_default()
                .into_iter()
                .map(|role| {
                    let bullets = by_role
                        .remove(&role.id)
                        .unwrap_or_default()
                        .into_iter()
                        .map(|bullet| BulletDetail {
                            skills: bskills.remove(&bullet.id).unwrap_or_default(),
                            variants: bvariants.remove(&bullet.id).unwrap_or_default(),
                            bullet,
                        })
                        .collect();
                    RoleDetail { role, bullets }
                })
                .collect();
            ExperienceDetail {
                skills: eskills.remove(&experience.id).unwrap_or_default(),
                roles,
                experience,
            }
        })
        .collect())
}
