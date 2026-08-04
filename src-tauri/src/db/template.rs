//! The `templates` table: built-in layouts plus whatever the user pasted in.

use crate::domain::Template;
use crate::error::{AppError, Result};
use sqlx::PgPool;
use uuid::Uuid;

const COLS: &str = "id, name, source, is_builtin, is_active";

pub async fn list(pool: &PgPool) -> Result<Vec<Template>> {
    Ok(sqlx::query_as::<_, Template>(&format!(
        "SELECT {COLS} FROM templates ORDER BY is_builtin DESC, name"
    ))
    .fetch_all(pool)
    .await?)
}

/// The template resumes are rendered with, or `None` before `sync_builtins` has run.
pub async fn get_active(pool: &PgPool) -> Result<Option<Template>> {
    Ok(
        sqlx::query_as::<_, Template>(&format!("SELECT {COLS} FROM templates WHERE is_active"))
            .fetch_optional(pool)
            .await?,
    )
}

pub async fn get(pool: &PgPool, id: Uuid) -> Result<Option<Template>> {
    Ok(
        sqlx::query_as::<_, Template>(&format!("SELECT {COLS} FROM templates WHERE id = $1"))
            .bind(id)
            .fetch_optional(pool)
            .await?,
    )
}

/// Writes a user template, replacing one of the same name.
///
/// Rejects a built-in name: startup re-syncs those from disk, so the edit would vanish on
/// the next launch.
pub async fn upsert(pool: &PgPool, name: &str, source: &str) -> Result<Template> {
    let name = name.trim();
    if name.is_empty() {
        return Err(AppError::Invalid("the template needs a name".into()));
    }
    let builtin: bool = sqlx::query_scalar(
        "SELECT COALESCE((SELECT is_builtin FROM templates WHERE name = $1), FALSE)",
    )
    .bind(name)
    .fetch_one(pool)
    .await?;
    if builtin {
        return Err(AppError::Invalid(format!(
            "{name} is built in — save your edit under a different name"
        )));
    }
    Ok(sqlx::query_as::<_, Template>(&format!(
        "INSERT INTO templates (name, source) VALUES ($1, $2)
         ON CONFLICT (name) DO UPDATE SET source = EXCLUDED.source, updated_at = now()
         RETURNING {COLS}"
    ))
    .bind(name)
    .bind(source)
    .fetch_one(pool)
    .await?)
}

/// Makes one template the one every render uses. Rejects an id that is not there.
pub async fn set_active(pool: &PgPool, id: Uuid) -> Result<()> {
    let n = sqlx::query(
        "UPDATE templates SET is_active = (id = $1)
         WHERE EXISTS (SELECT 1 FROM templates WHERE id = $1)",
    )
    .bind(id)
    .execute(pool)
    .await?
    .rows_affected();
    if n == 0 {
        return Err(AppError::NotFound("template".into()));
    }
    Ok(())
}

/// Rejects deleting a built-in — it would reappear on the next launch — and the active
/// one, which would leave nothing to render with.
pub async fn delete(pool: &PgPool, id: Uuid) -> Result<()> {
    let template = get(pool, id)
        .await?
        .ok_or_else(|| AppError::NotFound("template".into()))?;
    if template.is_builtin {
        return Err(AppError::Invalid(
            "built-in templates cannot be deleted".into(),
        ));
    }
    if template.is_active {
        return Err(AppError::Invalid(
            "select a different template before deleting this one".into(),
        ));
    }
    sqlx::query("DELETE FROM templates WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Copies the shipped templates into the table and makes sure something is active.
///
/// The files are the source of truth for built-ins, so this overwrites them on every
/// startup. A user row that happens to share a name is left alone.
pub async fn sync_builtins(pool: &PgPool, builtins: &[(&str, &str)]) -> Result<()> {
    for (name, source) in builtins {
        sqlx::query(
            "INSERT INTO templates (name, source, is_builtin) VALUES ($1, $2, TRUE)
             ON CONFLICT (name) DO UPDATE SET source = EXCLUDED.source, updated_at = now()
             WHERE templates.is_builtin",
        )
        .bind(name)
        .bind(source)
        .execute(pool)
        .await?;
    }
    sqlx::query(
        "UPDATE templates SET is_active = TRUE
         WHERE name = $1 AND NOT EXISTS (SELECT 1 FROM templates WHERE is_active)",
    )
    .bind(builtins.first().map(|(name, _)| *name).unwrap_or_default())
    .execute(pool)
    .await?;
    Ok(())
}
