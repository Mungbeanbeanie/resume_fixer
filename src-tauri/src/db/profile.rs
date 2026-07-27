//! The single-row `profile` table.

use crate::domain::Profile;
use crate::error::Result;
use sqlx::PgPool;

/// Returns the profile, or `None` before the user has filled one in.
pub async fn get(pool: &PgPool) -> Result<Option<Profile>> {
    Ok(
        sqlx::query_as::<_, Profile>("SELECT full_name, phone, email, links FROM profile")
            .fetch_optional(pool)
            .await?,
    )
}

/// Writes the profile, overwriting the single row if it exists.
pub async fn upsert(pool: &PgPool, p: &Profile) -> Result<Profile> {
    Ok(sqlx::query_as::<_, Profile>(
        "INSERT INTO profile (id, full_name, phone, email, links)
         VALUES (TRUE, $1, $2, $3, $4)
         ON CONFLICT (id) DO UPDATE
           SET full_name = EXCLUDED.full_name,
               phone     = EXCLUDED.phone,
               email     = EXCLUDED.email,
               links     = EXCLUDED.links,
               updated_at = now()
         RETURNING full_name, phone, email, links",
    )
    .bind(&p.full_name)
    .bind(&p.phone)
    .bind(&p.email)
    .bind(serde_json::to_value(&p.links).unwrap_or_else(|_| serde_json::json!([])))
    .fetch_one(pool)
    .await?)
}
