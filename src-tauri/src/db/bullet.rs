//! `bullets`, `bullet_variants`, and the candidate read that feeds retrieval.

use crate::domain::{Bullet, BulletInput, BulletVariant, Candidate};
use crate::error::{AppError, Result};
use sqlx::PgPool;
use std::collections::HashMap;
use uuid::Uuid;

const COLS: &str = "id, role_id, text, display_order, is_active";

pub async fn upsert(pool: &PgPool, input: &BulletInput) -> Result<Bullet> {
    if input.text.trim().is_empty() {
        return Err(AppError::Invalid("a bullet cannot be empty".into()));
    }
    Ok(sqlx::query_as::<_, Bullet>(&format!(
        "INSERT INTO bullets (id, role_id, text, display_order, is_active)
         VALUES (COALESCE($1, gen_random_uuid()), $2, $3, $4, $5)
         ON CONFLICT (id) DO UPDATE
           SET role_id = EXCLUDED.role_id, text = EXCLUDED.text,
               display_order = EXCLUDED.display_order, is_active = EXCLUDED.is_active,
               updated_at = now()
         RETURNING {COLS}"
    ))
    .bind(input.id)
    .bind(input.role_id)
    .bind(input.text.trim())
    .bind(input.display_order)
    .bind(input.is_active)
    .fetch_one(pool)
    .await?)
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<()> {
    let n = sqlx::query("DELETE FROM bullets WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    if n == 0 {
        return Err(AppError::NotFound("bullet".into()));
    }
    Ok(())
}

pub async fn get(pool: &PgPool, id: Uuid) -> Result<Bullet> {
    sqlx::query_as::<_, Bullet>(&format!("SELECT {COLS} FROM bullets WHERE id = $1"))
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or_else(|| AppError::NotFound("bullet".into()))
}

/// Records an alternate phrasing. `approved_at` stays NULL until the user accepts it.
pub async fn add_variant(
    pool: &PgPool,
    bullet_id: Uuid,
    text: &str,
    origin: &str,
) -> Result<BulletVariant> {
    Ok(sqlx::query_as::<_, BulletVariant>(
        "INSERT INTO bullet_variants (bullet_id, text, origin) VALUES ($1, $2, $3)
         RETURNING id, bullet_id, text, origin, approved_at",
    )
    .bind(bullet_id)
    .bind(text)
    .bind(origin)
    .fetch_one(pool)
    .await?)
}

/// Promotes a variant to `bullets.text`, archiving the wording it replaces.
///
/// The previous text becomes a `manual` variant, so accepting is never lossy.
pub async fn accept_variant(pool: &PgPool, variant_id: Uuid) -> Result<Bullet> {
    let mut tx = pool.begin().await?;
    let (bullet_id, text) = sqlx::query_as::<_, (Uuid, String)>(
        "SELECT bullet_id, text FROM bullet_variants WHERE id = $1",
    )
    .bind(variant_id)
    .fetch_optional(&mut *tx)
    .await?
    .ok_or_else(|| AppError::NotFound("variant".into()))?;

    let previous = sqlx::query_scalar::<_, String>("SELECT text FROM bullets WHERE id = $1")
        .bind(bullet_id)
        .fetch_one(&mut *tx)
        .await?;

    sqlx::query(
        "INSERT INTO bullet_variants (bullet_id, text, origin, approved_at)
                 VALUES ($1, $2, 'manual', now())",
    )
    .bind(bullet_id)
    .bind(previous)
    .execute(&mut *tx)
    .await?;

    sqlx::query("UPDATE bullet_variants SET approved_at = now() WHERE id = $1")
        .bind(variant_id)
        .execute(&mut *tx)
        .await?;

    let bullet = sqlx::query_as::<_, Bullet>(&format!(
        "UPDATE bullets SET text = $2, updated_at = now() WHERE id = $1 RETURNING {COLS}"
    ))
    .bind(bullet_id)
    .bind(&text)
    .fetch_one(&mut *tx)
    .await?;

    tx.commit().await?;
    Ok(bullet)
}

#[derive(sqlx::FromRow)]
struct CandidateRow {
    id: Uuid,
    text: String,
    role_id: Uuid,
    role_title: String,
    end_date: Option<chrono::NaiveDate>,
    experience_id: Uuid,
    org_name: String,
}

/// Every active bullet with the context retrieval scores against.
///
/// Rejects nothing — filtering by relevance is `pipeline::retrieval`'s job, not SQL's.
pub async fn candidates(pool: &PgPool) -> Result<Vec<Candidate>> {
    let rows = sqlx::query_as::<_, CandidateRow>(
        "SELECT b.id, b.text, r.id AS role_id, r.title AS role_title, r.end_date,
                e.id AS experience_id, e.org_name
         FROM bullets b
         JOIN roles r ON r.id = b.role_id
         JOIN experiences e ON e.id = r.experience_id
         WHERE b.is_active AND r.is_active AND e.is_active
         ORDER BY e.display_order, r.start_date DESC, b.display_order",
    )
    .fetch_all(pool)
    .await?;

    let bullet_slugs = sqlx::query_as::<_, (Uuid, String, f32)>(
        "SELECT bs.bullet_id, s.slug, bs.weight
         FROM bullet_skills bs JOIN skills s ON s.id = bs.skill_id",
    )
    .fetch_all(pool)
    .await?;

    let experience_slugs = sqlx::query_as::<_, (Uuid, String)>(
        "SELECT es.experience_id, s.slug
         FROM experience_skills es JOIN skills s ON s.id = es.skill_id",
    )
    .fetch_all(pool)
    .await?;

    let mut by_bullet: HashMap<Uuid, Vec<(String, f32)>> = HashMap::new();
    for (id, slug, weight) in bullet_slugs {
        by_bullet.entry(id).or_default().push((slug, weight));
    }
    let mut by_experience: HashMap<Uuid, Vec<String>> = HashMap::new();
    for (id, slug) in experience_slugs {
        by_experience.entry(id).or_default().push(slug);
    }

    Ok(rows
        .into_iter()
        .map(|r| Candidate {
            skills: by_bullet.remove(&r.id).unwrap_or_default(),
            experience_skills: by_experience
                .get(&r.experience_id)
                .cloned()
                .unwrap_or_default(),
            bullet_id: r.id,
            text: r.text,
            role_id: r.role_id,
            role_title: r.role_title,
            end_date: r.end_date,
            experience_id: r.experience_id,
            org_name: r.org_name,
            score: 0.0,
        })
        .collect())
}
