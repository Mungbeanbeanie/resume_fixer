//! The single error type crossing the IPC boundary.
//!
//! Serializes to `{ kind, message, detail? }` so the frontend branches on `kind`
//! instead of matching on message text.

use serde::Serialize;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("{0}")]
    Config(String),

    #[error("database error: {0}")]
    Db(#[from] sqlx::Error),

    #[error("migration failed: {0}")]
    Migrate(#[from] sqlx::migrate::MigrateError),

    #[error("{0} not found")]
    NotFound(String),

    #[error("{0}")]
    Invalid(String),

    #[error("could not fetch the posting: {0}")]
    Fetch(String),

    #[error("the page did not contain a readable job description")]
    ExtractFailed,

    #[error("the model call failed: {0}")]
    Llm(String),

    #[error("LaTeX compilation failed: {0}")]
    Render(String),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

impl AppError {
    /// Stable machine-readable discriminant. The frontend switches on this.
    pub fn kind(&self) -> &'static str {
        match self {
            AppError::Config(_) => "config",
            AppError::Db(_) | AppError::Migrate(_) => "db",
            AppError::NotFound(_) => "not_found",
            AppError::Invalid(_) => "invalid",
            AppError::Fetch(_) => "fetch",
            AppError::ExtractFailed => "extract_failed",
            AppError::Llm(_) => "llm",
            AppError::Render(_) => "render",
            AppError::Io(_) => "io",
        }
    }
}

#[derive(Serialize)]
struct Wire<'a> {
    kind: &'a str,
    message: String,
    detail: Option<String>,
}

impl Serialize for AppError {
    fn serialize<S: serde::Serializer>(&self, s: S) -> std::result::Result<S::Ok, S::Error> {
        let detail = match self {
            AppError::Db(e) => Some(format!("{e:?}")),
            AppError::Render(d) => Some(d.clone()),
            _ => None,
        };
        Wire {
            kind: self.kind(),
            message: self.to_string(),
            detail,
        }
        .serialize(s)
    }
}

pub type Result<T> = std::result::Result<T, AppError>;
