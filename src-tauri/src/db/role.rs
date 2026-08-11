//! `roles` — one experience can hold several.

use crate::domain::{Role, RoleInput};
use crate::error::{AppError, Result};
use sqlx::PgPool;
use uuid::Uuid;

const COLS: &str = "id, experience_id, title, location, start_date, end_date, date_override,
                    gpa, display_order, is_active";

pub async fn upsert(pool: &PgPool, input: &RoleInput) -> Result<Role> {
    Ok(sqlx::query_as::<_, Role>(&format!(
        "INSERT INTO roles (id, experience_id, title, location, start_date, end_date,
                            date_override, gpa, display_order, is_active)
         VALUES (COALESCE($1, gen_random_uuid()), $2, $3, $4, $5, $6, $7, $8, $9, $10)
         ON CONFLICT (id) DO UPDATE
           SET experience_id = EXCLUDED.experience_id, title = EXCLUDED.title,
               location = EXCLUDED.location, start_date = EXCLUDED.start_date,
               end_date = EXCLUDED.end_date, date_override = EXCLUDED.date_override,
               gpa = EXCLUDED.gpa,
               display_order = EXCLUDED.display_order, is_active = EXCLUDED.is_active
         RETURNING {COLS}"
    ))
    .bind(input.id)
    .bind(input.experience_id)
    .bind(&input.title)
    .bind(&input.location)
    .bind(input.start_date)
    .bind(input.end_date)
    .bind(&input.date_override)
    .bind(&input.gpa)
    .bind(input.display_order)
    .bind(input.is_active)
    .fetch_one(pool)
    .await?)
}

pub async fn delete(pool: &PgPool, id: Uuid) -> Result<()> {
    let n = sqlx::query("DELETE FROM roles WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?
        .rows_affected();
    if n == 0 {
        return Err(AppError::NotFound("role".into()));
    }
    Ok(())
}
