//! Connection pool construction and migration.

use crate::error::Result;
use sqlx::postgres::{PgPool, PgPoolOptions};

/// Builds the pool without requiring the server to be up yet.
///
/// Connecting eagerly would mean the app cannot open to tell the user their database is
/// down — which is the one thing the health strip exists to say.
pub fn lazy(url: &str) -> Result<PgPool> {
    Ok(PgPoolOptions::new().max_connections(5).connect_lazy(url)?)
}

/// Brings the schema up to date. Idempotent, so it is safe to call on every health check.
pub async fn migrate(pool: &PgPool) -> Result<()> {
    sqlx::migrate!("../migrations").run(pool).await?;
    Ok(())
}

/// True when the server answers and the schema is current.
pub async fn is_ready(pool: &PgPool) -> bool {
    sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(pool)
        .await
        .is_ok()
        && migrate(pool).await.is_ok()
}
