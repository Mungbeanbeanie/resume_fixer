//! The `base_resumes` table: resumes rendered from the whole vault, with no job behind
//! them.

use crate::domain::BaseResume;
use crate::error::{AppError, Result};
use sqlx::PgPool;
use uuid::Uuid;

const COLS: &str = "id, name, template_name, pdf_path, page_count, created_at";

pub async fn list(pool: &PgPool) -> Result<Vec<BaseResume>> {
    Ok(sqlx::query_as::<_, BaseResume>(&format!(
        "SELECT {COLS} FROM base_resumes ORDER BY created_at DESC"
    ))
    .fetch_all(pool)
    .await?)
}

pub async fn get(pool: &PgPool, id: Uuid) -> Result<Option<BaseResume>> {
    Ok(
        sqlx::query_as::<_, BaseResume>(&format!("SELECT {COLS} FROM base_resumes WHERE id = $1"))
            .bind(id)
            .fetch_optional(pool)
            .await?,
    )
}

/// Stores the rendered `.tex` alongside the name so a saved base resume stays readable
/// even after its template is edited or deleted.
pub async fn insert(
    pool: &PgPool,
    name: &str,
    template_name: &str,
    tex_source: &str,
    page_count: i32,
) -> Result<BaseResume> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::Invalid("name this base resume first".into()));
    }
    Ok(sqlx::query_as::<_, BaseResume>(&format!(
        "INSERT INTO base_resumes (name, template_name, tex_source, page_count)
         VALUES ($1, $2, $3, $4) RETURNING {COLS}"
    ))
    .bind(name)
    .bind(template_name)
    .bind(tex_source)
    .bind(page_count)
    .fetch_one(pool)
    .await?)
}

pub async fn set_pdf_path(pool: &PgPool, id: Uuid, path: &str) -> Result<()> {
    sqlx::query("UPDATE base_resumes SET pdf_path = $2 WHERE id = $1")
        .bind(id)
        .bind(path)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<()> {
    let n = sqlx::query("DELETE FROM base_resumes WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    if n == 0 {
        return Err(AppError::NotFound("base resume".into()));
    }
    Ok(())
}
