//! Structs shared by the repositories, the pipeline, and the IPC boundary.
//!
//! `src/types.ts` mirrors this file. Change one, change both in the same commit.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "experience_kind", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ExperienceKind {
    Work,
    Project,
    Education,
    Certification,
    Activity,
}

/// In the order an application moves through them, which is the order the dropdown offers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "application_status", rename_all = "snake_case")]
#[serde(rename_all = "snake_case")]
pub enum ApplicationStatus {
    Saved,
    Applied,
    OaReceived,
    OaCompleted,
    // The round numbers need spelling out: neither serde nor sqlx puts an underscore before
    // a digit, and the database values do.
    #[sqlx(rename = "interview_1")]
    #[serde(rename = "interview_1")]
    Interview1,
    #[sqlx(rename = "interview_2")]
    #[serde(rename = "interview_2")]
    Interview2,
    #[sqlx(rename = "interview_3")]
    #[serde(rename = "interview_3")]
    Interview3,
    Offer,
    Rejected,
    Withdrawn,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "job_source", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum JobSource {
    Fetched,
    Pasted,
}

// ── Profile ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Link {
    pub label: String,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Profile {
    pub full_name: String,
    pub phone: Option<String>,
    pub email: Option<String>,
    #[sqlx(json)]
    pub links: Vec<Link>,
    /// Comma-separated, printed verbatim by the templates that name `interests`.
    pub interests: Option<String>,
}

// ── Vault ───────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Experience {
    pub id: Uuid,
    pub kind: ExperienceKind,
    pub org_name: String,
    pub location: Option<String>,
    pub url: Option<String>,
    pub tech_line: Option<String>,
    pub display_order: i32,
    pub is_active: bool,
    /// The fit loop may never retire this experience, however weakly it scores.
    pub is_pinned: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Role {
    pub id: Uuid,
    pub experience_id: Uuid,
    pub title: String,
    pub location: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub date_override: Option<String>,
    /// Free text, printed after the degree on education entries. Free rather than numeric
    /// so "3.87/4.00" and "3.9 (Major: 4.0)" both survive to the page as written.
    pub gpa: Option<String>,
    pub display_order: i32,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Bullet {
    pub id: Uuid,
    pub role_id: Uuid,
    pub text: String,
    pub display_order: i32,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Skill {
    pub id: Uuid,
    pub name: String,
    pub slug: String,
    pub category: Option<String>,
    pub aliases: Vec<String>,
    /// Prints on the base resume even when nothing in the vault tags it.
    pub always_list: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct BulletVariant {
    pub id: Uuid,
    pub bullet_id: Uuid,
    pub text: String,
    pub origin: String,
    pub approved_at: Option<DateTime<Utc>>,
}

/// A bullet with the skills tagged on it.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BulletDetail {
    #[serde(flatten)]
    pub bullet: Bullet,
    pub skills: Vec<Skill>,
    pub variants: Vec<BulletVariant>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleDetail {
    #[serde(flatten)]
    pub role: Role,
    pub bullets: Vec<BulletDetail>,
}

/// The whole tree for one experience — what the Vault tab renders per accordion row.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperienceDetail {
    #[serde(flatten)]
    pub experience: Experience,
    pub skills: Vec<Skill>,
    pub roles: Vec<RoleDetail>,
}

// ── Vault write inputs ──────────────────────────────────────────────────

#[derive(Debug, Clone, Deserialize)]
pub struct ExperienceInput {
    pub id: Option<Uuid>,
    pub kind: ExperienceKind,
    pub org_name: String,
    pub location: Option<String>,
    pub url: Option<String>,
    pub tech_line: Option<String>,
    pub display_order: i32,
    pub is_active: bool,
    /// The fit loop may never retire this experience, however weakly it scores.
    pub is_pinned: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RoleInput {
    pub id: Option<Uuid>,
    pub experience_id: Uuid,
    pub title: String,
    pub location: Option<String>,
    pub start_date: NaiveDate,
    pub end_date: Option<NaiveDate>,
    pub date_override: Option<String>,
    pub gpa: Option<String>,
    pub display_order: i32,
    pub is_active: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BulletInput {
    pub id: Option<Uuid>,
    pub role_id: Uuid,
    pub text: String,
    pub display_order: i32,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct Suggestion {
    pub variant_id: Uuid,
    pub text: String,
}

/// One line of a draft as the user wants it printed.
///
/// `keep: false` drops the line from this resume only — the bullet stays in the vault.
#[derive(Debug, Clone, Deserialize)]
pub struct BulletEdit {
    pub bullet_id: Uuid,
    pub text: String,
    pub keep: bool,
}

/// One active bullet with everything retrieval scores against.
#[derive(Debug, Clone, Serialize)]
pub struct Candidate {
    pub bullet_id: Uuid,
    pub text: String,
    pub role_id: Uuid,
    pub role_title: String,
    pub end_date: Option<NaiveDate>,
    pub experience_id: Uuid,
    pub org_name: String,
    /// `(slug, weight)` from `bullet_skills`.
    pub skills: Vec<(String, f32)>,
    /// Slugs from `experience_skills`, shared by every bullet under the experience.
    pub experience_skills: Vec<String>,
    pub score: f32,
}

// ── Library ─────────────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Application {
    pub id: Uuid,
    pub url: Option<String>,
    pub company: Option<String>,
    pub role_title: Option<String>,
    pub job_text: String,
    pub job_source: JobSource,
    pub parsed: serde_json::Value,
    pub status: ApplicationStatus,
    pub applied_at: Option<DateTime<Utc>>,
    pub notes: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct ApplicationSummary {
    pub id: Uuid,
    pub company: Option<String>,
    pub role_title: Option<String>,
    pub url: Option<String>,
    pub status: ApplicationStatus,
    pub created_at: DateTime<Utc>,
    pub applied_at: Option<DateTime<Utc>>,
    pub resume_id: Option<Uuid>,
    pub pdf_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Resume {
    pub id: Uuid,
    pub application_id: Uuid,
    pub tex_source: String,
    pub pdf_path: Option<String>,
    pub model: String,
    pub prompt_version: String,
    pub feedback: Option<String>,
    pub page_count: Option<i32>,
    pub is_current: bool,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct StatusChange {
    pub status: ApplicationStatus,
    pub changed_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ApplicationDetail {
    #[serde(flatten)]
    pub application: Application,
    pub resume: Option<Resume>,
    pub history: Vec<StatusChange>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ApplicationPatch {
    pub company: Option<String>,
    pub role_title: Option<String>,
    pub notes: Option<String>,
}

/// Counts behind the Library graph, keyed by status. A status nobody is sitting on is
/// absent rather than zero. A map rather than a field per status: the set of statuses moves,
/// and the graph decides how to group them.
pub type StatusStats = HashMap<ApplicationStatus, i64>;

// ── Pipeline results ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct IngestResult {
    pub text: String,
    pub source: JobSource,
    pub needs_paste: bool,
    /// Why the fetch or the extraction gave up, when it did. A login wall, a timeout and a
    /// JavaScript-rendered page all need different things from the user.
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UsedBullet {
    pub bullet_id: Uuid,
    pub source_text: String,
    pub rendered_text: String,
    pub was_reworded: bool,
    pub org_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RejectedRewrite {
    pub bullet_id: Uuid,
    pub attempted: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GenerationResult {
    pub draft_id: Uuid,
    pub pdf_path: String,
    pub page_count: i32,
    pub company: Option<String>,
    pub role_title: Option<String>,
    pub used_bullets: Vec<UsedBullet>,
    pub rejected: Vec<RejectedRewrite>,
    pub dropped_for_fit: i32,
    /// Experiences the fit loop retired whole, named so the user is never silently edited.
    pub retired: Vec<String>,
}

// ── Templates and base resumes ──────────────────────────────────────────

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct Template {
    pub id: Uuid,
    pub name: String,
    pub source: String,
    pub is_builtin: bool,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, sqlx::FromRow)]
pub struct BaseResume {
    pub id: Uuid,
    pub name: String,
    pub template_name: String,
    pub pdf_path: Option<String>,
    pub page_count: Option<i32>,
    pub created_at: DateTime<Utc>,
}

/// A base resume that has been compiled but not saved. Mirrors the draft rule: nothing
/// reaches the database until the user names it.
#[derive(Debug, Clone, Serialize)]
pub struct BasePreview {
    pub template_name: String,
    pub pdf_path: String,
    pub page_count: i32,
    pub bullet_count: i32,
    /// The printed lines, so the preview can be trimmed by hand before it is saved.
    pub used_bullets: Vec<UsedBullet>,
    /// Experiences the fit loop retired whole, named so the user is never silently edited.
    pub retired: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct HealthReport {
    pub postgres: bool,
    pub ollama: bool,
    pub model_present: bool,
    pub tectonic: bool,
    pub model: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The database values are what the two enums have to agree on, and a typo in either
    /// only shows up as a decode error at runtime.
    #[test]
    fn every_status_serializes_to_its_database_value() {
        let names: Vec<String> = [
            ApplicationStatus::Saved,
            ApplicationStatus::Applied,
            ApplicationStatus::OaReceived,
            ApplicationStatus::OaCompleted,
            ApplicationStatus::Interview1,
            ApplicationStatus::Interview2,
            ApplicationStatus::Interview3,
            ApplicationStatus::Offer,
            ApplicationStatus::Rejected,
            ApplicationStatus::Withdrawn,
        ]
        .iter()
        .map(|s| serde_json::to_string(s).expect("a plain enum serializes"))
        .collect();
        assert_eq!(
            names.join(","),
            r#""saved","applied","oa_received","oa_completed","interview_1","interview_2","interview_3","offer","rejected","withdrawn""#
        );
    }
}
