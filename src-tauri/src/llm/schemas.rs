//! The JSON shapes the model is allowed to return. Anything else is a failed call.

use serde::Deserialize;
use uuid::Uuid;

#[derive(Debug, Clone, Default, Deserialize, serde::Serialize)]
pub struct ParsedJob {
    #[serde(default)]
    pub company: Option<String>,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub hard_skills: Vec<String>,
    #[serde(default)]
    pub soft_signals: Vec<String>,
    #[serde(default)]
    pub domain: Option<String>,
    #[serde(default)]
    pub seniority: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SelectedBullet {
    pub id: Uuid,
    #[serde(default)]
    pub rewrite: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SelectedSection {
    pub experience_id: Uuid,
    #[serde(default)]
    pub bullets: Vec<SelectedBullet>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Selection {
    #[serde(default)]
    pub sections: Vec<SelectedSection>,
    #[serde(default)]
    pub skills_line: Vec<String>,
    #[serde(default)]
    pub reasoning: Option<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct Improvements {
    #[serde(default)]
    pub suggestions: Vec<String>,
}
