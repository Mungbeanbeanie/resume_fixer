//! `applications`, its status history, and the Library counts.

use crate::domain::*;
use crate::error::{AppError, Result};
use sqlx::PgPool;
use uuid::Uuid;

const COLS: &str = "id, url, company, role_title, job_text, job_source, parsed, status,
                    applied_at, notes, created_at";

/// Creates an application and its opening history row in one transaction.
// Every column of the row is required at insert time; a builder would only move the
// argument list somewhere else.
#[allow(clippy::too_many_arguments)]
pub async fn create(
    pool: &PgPool,
    url: Option<&str>,
    company: Option<&str>,
    role_title: Option<&str>,
    job_text: &str,
    job_source: JobSource,
    parsed: &serde_json::Value,
    status: ApplicationStatus,
) -> Result<Application> {
    let mut tx = pool.begin().await?;
    // Same rule `set_status` uses: anything past `saved` has been sent, and one tracked
    // after the fact often opens at a later status than `applied`.
    let applied_at = (!matches!(status, ApplicationStatus::Saved)).then(chrono::Utc::now);
    let app = sqlx::query_as::<_, Application>(&format!(
        "INSERT INTO applications (url, company, role_title, job_text, job_source, parsed,
                                   status, applied_at)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8) RETURNING {COLS}"
    ))
    .bind(url)
    .bind(company)
    .bind(role_title)
    .bind(job_text)
    .bind(job_source)
    .bind(parsed)
    .bind(status)
    .bind(applied_at)
    .fetch_one(&mut *tx)
    .await?;

    sqlx::query("INSERT INTO application_status_history (application_id, status) VALUES ($1, $2)")
        .bind(app.id)
        .bind(status)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(app)
}

/// Table rows, newest first, optionally narrowed to one status.
pub async fn list_summaries(
    pool: &PgPool,
    filter: Option<ApplicationStatus>,
) -> Result<Vec<ApplicationSummary>> {
    Ok(sqlx::query_as::<_, ApplicationSummary>(
        "SELECT a.id, a.company, a.role_title, a.url, a.status, a.created_at, a.applied_at,
                r.id AS resume_id, r.pdf_path
         FROM applications a
         LEFT JOIN resumes r ON r.application_id = a.id AND r.is_current
         WHERE $1::application_status IS NULL OR a.status = $1
         ORDER BY a.created_at DESC",
    )
    .bind(filter)
    .fetch_all(pool)
    .await?)
}

pub async fn get_detail(pool: &PgPool, id: Uuid) -> Result<ApplicationDetail> {
    let application =
        sqlx::query_as::<_, Application>(&format!("SELECT {COLS} FROM applications WHERE id = $1"))
            .bind(id)
            .fetch_optional(pool)
            .await?
            .ok_or_else(|| AppError::NotFound("application".into()))?;

    let resume = super::resume::current_for(pool, id).await?;
    let history = sqlx::query_as::<_, StatusChange>(
        "SELECT status, changed_at FROM application_status_history
         WHERE application_id = $1 ORDER BY changed_at",
    )
    .bind(id)
    .fetch_all(pool)
    .await?;

    Ok(ApplicationDetail {
        application,
        resume,
        history,
    })
}

/// Moves an application to a new status and appends a history row.
///
/// `applied_at` is stamped the first time the application reaches `applied` and never
/// moved afterwards — a later rejection must not rewrite when it was sent.
pub async fn set_status(pool: &PgPool, id: Uuid, status: ApplicationStatus) -> Result<()> {
    let mut tx = pool.begin().await?;
    let n = sqlx::query(
        "UPDATE applications
         SET status = $2,
             applied_at = CASE WHEN applied_at IS NULL AND $2 <> 'saved' THEN now()
                               ELSE applied_at END,
             updated_at = now()
         WHERE id = $1",
    )
    .bind(id)
    .bind(status)
    .execute(&mut *tx)
    .await?
    .rows_affected();
    if n == 0 {
        return Err(AppError::NotFound("application".into()));
    }
    sqlx::query("INSERT INTO application_status_history (application_id, status) VALUES ($1, $2)")
        .bind(id)
        .bind(status)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(())
}

/// Patches the editable fields. A `None` field leaves the stored value alone.
pub async fn update(pool: &PgPool, id: Uuid, patch: &ApplicationPatch) -> Result<()> {
    sqlx::query(
        "UPDATE applications
         SET company    = COALESCE($2, company),
             role_title = COALESCE($3, role_title),
             notes      = COALESCE($4, notes),
             updated_at = now()
         WHERE id = $1",
    )
    .bind(id)
    .bind(&patch.company)
    .bind(&patch.role_title)
    .bind(&patch.notes)
    .execute(pool)
    .await?;
    Ok(())
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<()> {
    let n = sqlx::query("DELETE FROM applications WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    if n == 0 {
        return Err(AppError::NotFound("application".into()));
    }
    Ok(())
}

pub async fn stats(pool: &PgPool) -> Result<StatusStats> {
    Ok(sqlx::query_as::<_, (ApplicationStatus, i64)>(
        "SELECT status, count(*) FROM applications GROUP BY status",
    )
    .fetch_all(pool)
    .await?
    .into_iter()
    .collect())
}
