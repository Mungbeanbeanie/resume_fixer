//! Orchestration. Services call repositories, the model, and the renderer — never sqlx
//! types directly.

pub mod base;
pub mod generate;
pub mod library;
pub mod vault;
