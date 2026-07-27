//! Structs shared by the repositories, the pipeline, and the IPC boundary.
//!
//! `src/types.ts` mirrors this file. Change one, change both in the same commit.

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "experience_kind", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ExperienceKind {
    Work,
    Project,
    Education,
    Certification,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "application_status", rename_all = "lowercase")]
#[serde(rename_all = "lowercase")]
pub enum ApplicationStatus {
    Saved,
    Applied,
    Rejected,
    Interview,
    Offer,
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

/// Counts behind the Library graph. `saved` and `withdrawn` were never sent, so they
/// are reported separately and excluded from the bar.
#[derive(Debug, Clone, Default, Serialize)]
pub struct StatusStats {
    pub saved: i64,
    pub applied: i64,
    pub interview: i64,
    pub offer: i64,
    pub rejected: i64,
    pub withdrawn: i64,
}

impl StatusStats {
    /// Share of sent applications that got any answer, interview/offer/rejection alike.
    pub fn response_rate(&self) -> f32 {
        let sent = self.applied + self.interview + self.offer + self.rejected;
        if sent == 0 {
            return 0.0;
        }
        (self.interview + self.offer + self.rejected) as f32 / sent as f32
    }
}

// ── Pipeline results ────────────────────────────────────────────────────

#[derive(Debug, Clone, Serialize)]
pub struct IngestResult {
    pub text: String,
    pub source: JobSource,
    pub needs_paste: bool,
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

    #[test]
    fn response_rate_ignores_unsent() {
        let s = StatusStats {
            saved: 10,
            applied: 2,
            interview: 1,
            rejected: 1,
            ..Default::default()
        };
        assert_eq!(s.response_rate(), 0.5);
        assert_eq!(StatusStats::default().response_rate(), 0.0);
    }
}
