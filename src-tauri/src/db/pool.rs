//! Connection pool construction and migration.

use crate::error::Result;
use sqlx::migrate::MigrateDatabase;
use sqlx::postgres::{PgConnectOptions, PgPool, PgPoolOptions};
use sqlx::Postgres;
use std::str::FromStr;

/// Builds the pool without requiring the server to be up yet.
///
/// Connecting eagerly would mean the app cannot open to tell the user their database is
/// down — which is the one thing the health strip exists to say.
///
/// `lock_timeout` is three seconds because a migration that fails partway leaves sqlx's
/// advisory lock held by that connection. Without a bound, every later `migrate` call
/// queues on it and the health check never returns, so the strip reads "checking" forever
/// instead of reporting the fault. Failing the wait turns a silent hang into the red dot.
pub fn lazy(url: &str) -> Result<PgPool> {
    let options = PgConnectOptions::from_str(url)?.options([("lock_timeout", "3000")]);
    Ok(PgPoolOptions::new()
        .max_connections(5)
        .connect_lazy_with(options))
}

/// Brings the schema up to date. Idempotent, so it is safe to call on every health check.
///
/// Applied migrations with no file behind them are accepted. Versions 2, 7, 8 and 10 seeded
/// the maintainer's own vault and live in `seed/`, applied by hand, so a database written
/// before that move records them and would otherwise fail with `VersionMissing`.
pub async fn migrate(pool: &PgPool) -> Result<()> {
    let mut migrator = sqlx::migrate!("../migrations");
    migrator.set_ignore_missing(true);
    migrator.run(pool).await?;
    Ok(())
}

/// True when the server answers and the schema is current.
///
/// Creates the database when it is absent, which is the state every fresh install starts
/// in: the app ships an empty vault, and a user whose server is running should not have to
/// find a terminal to run `createdb`. The attempt is made only once `SELECT 1` has failed,
/// so an existing database costs nothing extra, and a create that cannot succeed — no
/// server, or a role without CREATE DATABASE — is the red dot rather than an error.
pub async fn is_ready(pool: &PgPool, url: &str) -> bool {
    if sqlx::query_scalar::<_, i32>("SELECT 1")
        .fetch_one(pool)
        .await
        .is_err()
        && Postgres::create_database(url).await.is_err()
    {
        return false;
    }
    // The pool connects lazily and caches no failure, so this reaches the new database.
    migrate(pool).await.is_ok()
}
