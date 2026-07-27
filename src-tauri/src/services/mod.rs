//! Orchestration. Services call repositories, the model, and the renderer — never sqlx
//! types directly.

pub mod generate;
pub mod vault;
