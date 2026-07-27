//! `resumes` and the `resume_bullets` provenance rows.

use crate::domain::{Resume, UsedBullet};
use crate::error::Result;
use sqlx::PgPool;
use uuid::Uuid;

const COLS: &str = "id, application_id, tex_source, pdf_path, model, prompt_version, feedback,
                    page_count, is_current, created_at";

/// Stores a rendered resume together with the bullet rows that produced every line.
///
/// Rejects nothing, but the `resume_bullets` foreign key does: a line whose `bullet_id`
/// is not in `bullets` cannot be written, which is the structural half of the
/// anti-hallucination guarantee.
// Every column of the row is required at insert time; a builder would only move the
// argument list somewhere else.
#[allow(clippy::too_many_arguments)]
pub async fn insert(
    pool: &PgPool,
    application_id: Uuid,
    tex_source: &str,
    pdf_path: Option<&str>,
    model: &str,
    prompt_version: &str,
    feedback: Option<&str>,
    page_count: i32,
    used: &[UsedBullet],
) -> Result<Resume> {
    let mut tx = pool.begin().await?;
    sqlx::query("UPDATE resumes SET is_current = FALSE WHERE application_id = $1")
        .bind(application_id)
        .execute(&mut *tx)
        .await?;

    let resume = sqlx::query_as::<_, Resume>(&format!(
        "INSERT INTO resumes (application_id, tex_source, pdf_path, model, prompt_version,
                              feedback, page_count)
         VALUES ($1, $2, $3, $4, $5, $6, $7) RETURNING {COLS}"
    ))
    .bind(application_id)
    .bind(tex_source)
    .bind(pdf_path)
    .bind(model)
    .bind(prompt_version)
    .bind(feedback)
    .bind(page_count)
    .fetch_one(&mut *tx)
    .await?;

    for (i, u) in used.iter().enumerate() {
        sqlx::query(
            "INSERT INTO resume_bullets (resume_id, bullet_id, rendered_text, was_reworded,
                                         display_order)
             VALUES ($1, $2, $3, $4, $5)
             ON CONFLICT (resume_id, bullet_id) DO NOTHING",
        )
        .bind(resume.id)
        .bind(u.bullet_id)
        .bind(&u.rendered_text)
        .bind(u.was_reworded)
        .bind(i as i32)
        .execute(&mut *tx)
        .await?;
    }
    tx.commit().await?;
    Ok(resume)
}

pub async fn current_for(pool: &PgPool, application_id: Uuid) -> Result<Option<Resume>> {
    Ok(sqlx::query_as::<_, Resume>(&format!(
        "SELECT {COLS} FROM resumes WHERE application_id = $1 AND is_current"
    ))
    .bind(application_id)
    .fetch_optional(pool)
    .await?)
}

pub async fn get(pool: &PgPool, id: Uuid) -> Result<Option<Resume>> {
    Ok(
        sqlx::query_as::<_, Resume>(&format!("SELECT {COLS} FROM resumes WHERE id = $1"))
            .bind(id)
            .fetch_optional(pool)
            .await?,
    )
}
