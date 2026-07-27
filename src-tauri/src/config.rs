//! `~/.config/resume-fixer/config.toml`, written with defaults on first run.
//!
//! The file is the single source of truth; the only values hardcoded here are the
//! defaults used to write it.

use crate::error::{AppError, Result};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub database: Database,
    pub llm: Llm,
    pub render: Render,
    pub ingest: Ingest,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Database {
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Llm {
    pub endpoint: String,
    pub model: String,
    pub timeout_secs: u64,
    pub temperature: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Render {
    pub tectonic_path: String,
    pub output_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Ingest {
    pub timeout_secs: u64,
    pub min_chars: usize,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            database: Database {
                url: "postgresql://localhost/resume_fixer".into(),
            },
            llm: Llm {
                endpoint: "http://localhost:11434".into(),
                model: "ornith:35b".into(),
                timeout_secs: 180,
                // Low: selecting and compressing existing bullets is not a creative task.
                temperature: 0.2,
            },
            render: Render {
                tectonic_path: "tectonic".into(),
                output_dir: "~/Documents/Resumes".into(),
            },
            ingest: Ingest {
                timeout_secs: 10,
                min_chars: 400,
            },
        }
    }
}

fn home() -> Result<PathBuf> {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| AppError::Config("HOME is not set".into()))
}

/// Expands a leading `~/` against `$HOME`. Leaves every other path untouched.
pub fn expand_tilde(path: &str) -> Result<PathBuf> {
    match path.strip_prefix("~/") {
        Some(rest) => Ok(home()?.join(rest)),
        None => Ok(PathBuf::from(path)),
    }
}

pub fn config_path() -> Result<PathBuf> {
    Ok(home()?.join(".config/resume-fixer/config.toml"))
}

impl Config {
    /// Reads the config file, creating it with defaults if it is absent.
    ///
    /// Rejects a file that exists but does not parse — silently falling back to
    /// defaults would point the app at the wrong database.
    pub fn load() -> Result<Config> {
        let path = config_path()?;
        if !path.exists() {
            let cfg = Config::default();
            if let Some(dir) = path.parent() {
                std::fs::create_dir_all(dir)?;
            }
            std::fs::write(
                &path,
                toml::to_string_pretty(&cfg)
                    .map_err(|e| AppError::Config(format!("could not serialize defaults: {e}")))?,
            )?;
            return Ok(cfg);
        }
        let text = std::fs::read_to_string(&path)?;
        toml::from_str(&text)
            .map_err(|e| AppError::Config(format!("{} is malformed: {e}", path.display())))
    }

    /// Directory downloads land in, tilde expanded and created if missing.
    pub fn output_dir(&self) -> Result<PathBuf> {
        let dir = expand_tilde(&self.render.output_dir)?;
        std::fs::create_dir_all(&dir)?;
        Ok(dir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_round_trip_through_toml() {
        let text = toml::to_string_pretty(&Config::default()).unwrap();
        let back: Config = toml::from_str(&text).unwrap();
        assert_eq!(back.llm.model, "ornith:35b");
        assert_eq!(back.ingest.min_chars, 400);
    }

    #[test]
    fn tilde_expands_only_at_the_front() {
        std::env::set_var("HOME", "/home/x");
        assert_eq!(
            expand_tilde("~/Docs").unwrap(),
            PathBuf::from("/home/x/Docs")
        );
        assert_eq!(expand_tilde("/abs/~/y").unwrap(), PathBuf::from("/abs/~/y"));
    }
}
