//! IPC handlers, one file per tab. Thin by rule: they translate arguments and call a
//! service or a repository.

pub mod base;
pub mod generate;
pub mod health;
pub mod library;
pub mod vault;
