//! The generation pipeline, end to end.
//!
//! Ingest → parse → retrieve → select → ground → render → fit. Stages 1, 3, 5 and 6 are
//! deterministic; a failure in either model stage degrades the result instead of ending it.

use crate::db;
use crate::domain::*;
use crate::error::{AppError, Result};
use crate::llm::{self, schemas::ParsedJob, PROMPT_VERSION};
use crate::pipeline::{fit, retrieval, select, skills::SkillIndex};
use crate::render::templates::DEFAULT_TEMPLATE;
use crate::render::{tectonic, tex};
use crate::state::{AppState, Draft};
use uuid::Uuid;

/// Reads the posting without a model: any vault skill whose slug or alias appears in the
/// text is treated as wanted. Cruder than the model, but it never fails.
fn keyword_scan(job_text: &str, all: &[Skill]) -> ParsedJob {
    let lower = job_text.to_lowercase();
    let hard_skills = all
        .iter()
        .filter(|s| {
            let mut forms = vec![s.name.to_lowercase(), s.slug.clone()];
            forms.extend(s.aliases.iter().map(|a| a.to_lowercase()));
            forms
                .iter()
                .any(|f| f.len() > 1 && lower.contains(f.as_str()))
        })
        .map(|s| s.name.clone())
        .collect();
    ParsedJob {
        hard_skills,
        ..Default::default()
    }
}

/// What the posting is about, for `retrieval`'s topical signal.
///
/// The parser already reports these; nothing scored them, so a policy team and an agent
/// team ranked the vault identically as long as they named the same languages. The title
/// counts because it is often the only place the domain appears at all.
fn topics(parsed: &ParsedJob) -> Vec<String> {
    parsed
        .domain
        .iter()
        .chain(parsed.title.iter())
        .cloned()
        .chain(parsed.soft_signals.iter().cloned())
        .collect()
}

/// Runs the whole pipeline and parks the result as an in-memory draft.
///
/// Rejects an empty job description — everything downstream would be guesswork.
pub async fn generate(
    state: &AppState,
    job_text: &str,
    url: Option<String>,
    job_source: JobSource,
    feedback: Option<String>,
) -> Result<GenerationResult> {
    if job_text.trim().len() < 40 {
        return Err(AppError::Invalid(
            "paste the job description first — there is not enough here to tailor to".into(),
        ));
    }

    let all_skills = db::skill::list(&state.pool).await?;
    let index = SkillIndex::new(&all_skills);

    let parsed = match llm::parse_job(state.llm.as_ref(), job_text).await {
        Ok(p) => p,
        Err(e) => {
            tracing::warn!("job parsing fell back to a keyword scan: {e}");
            keyword_scan(job_text, &all_skills)
        }
    };

    let profile = db::profile::get(&state.pool).await?;
    let vault = db::experience::list_details(&state.pool).await?;
    // `candidates` comes back scored and sorted: the shortlist is what the model reads, the
    // whole set is what fills out the entries it chose.
    let mut candidates = db::bullet::candidates(&state.pool).await?;
    if candidates.is_empty() {
        return Err(AppError::Invalid(
            "the vault is empty — add an experience before generating".into(),
        ));
    }
    let shortlist = retrieval::shortlist(
        &mut candidates,
        &parsed.hard_skills,
        &topics(&parsed),
        &index,
    );

    let outcome = match select::run(
        state.llm.as_ref(),
        &parsed,
        &shortlist,
        &candidates,
        &vault,
        profile.clone(),
        &index,
        &all_skills,
        feedback.as_deref(),
    )
    .await
    {
        Ok(o) => o,
        Err(e) => {
            tracing::warn!("selection fell back to deterministic scoring: {e}");
            select::deterministic(
                &shortlist,
                &candidates,
                &vault,
                profile,
                &index,
                &parsed.hard_skills,
            )
        }
    };

    // The template the user selected in the Base tab, or the built-in on a database that
    // has not synced yet.
    let template = db::template::get_active(&state.pool)
        .await?
        .map(|t| t.source)
        .unwrap_or_else(|| DEFAULT_TEMPLATE.to_string());

    let mut plan = outcome.plan;
    plan.prune_for(&template);
    plan.cap_bullets();
    let draft_id = Uuid::new_v4();
    let workdir = state.draft_dir(draft_id);
    let fitted = fit::fit_to_one_page(
        &state.config.render.tectonic_path,
        &template,
        &mut plan,
        &workdir,
    )
    .await?;

    let result = GenerationResult {
        draft_id,
        pdf_path: fitted.pdf_path.to_string_lossy().to_string(),
        page_count: fitted.page_count,
        company: parsed.company.clone(),
        role_title: parsed.title.clone(),
        used_bullets: plan.used_bullets(),
        rejected: outcome.rejected.clone(),
        dropped_for_fit: fitted.dropped,
        retired: fitted.retired,
    };

    state.drafts.lock().await.insert(
        draft_id,
        Draft {
            plan,
            tex: fitted.tex,
            pdf_path: fitted.pdf_path,
            page_count: fitted.page_count,
            job_text: job_text.to_string(),
            url,
            job_source,
            parsed: serde_json::to_value(&parsed).unwrap_or_default(),
            company: parsed.company,
            role_title: parsed.title,
            rejected: outcome.rejected,
            feedback,
        },
    );
    Ok(result)
}

/// Reworks a draft to the user's own edits and recompiles it.
///
/// No model runs, no fit loop, and no bullet cap: the user is fine-tuning, and paying for
/// their edit by silently dropping someone else's bullet — or the one they just added —
/// would undo the thing they asked for. An overlong result comes back with its real page
/// count for the caller to warn about. Rejects an edit set that would empty the resume.
pub async fn revise(
    state: &AppState,
    draft_id: Uuid,
    edits: Vec<BulletEdit>,
) -> Result<GenerationResult> {
    let template = db::template::get_active(&state.pool)
        .await?
        .map(|t| t.source)
        .unwrap_or_else(|| DEFAULT_TEMPLATE.to_string());
    // An edit may name a bullet this draft never printed; the vault is where its text lives.
    let vault = db::experience::list_details(&state.pool).await?;

    let mut drafts = state.drafts.lock().await;
    let draft = drafts
        .get_mut(&draft_id)
        .ok_or_else(|| AppError::NotFound("draft".into()))?;

    draft.plan.apply_edits(&vault, &edits);
    draft.plan.prune_for(&template);
    if draft.plan.bullet_count() == 0 {
        return Err(AppError::Invalid(
            "that would leave the resume with no bullets — keep at least one".into(),
        ));
    }

    let tex_source = tex::render(&template, &draft.plan.to_render_input())?;
    let compiled = tectonic::compile(
        &state.config.render.tectonic_path,
        &tex_source,
        &state.draft_dir(draft_id),
    )
    .await?;

    draft.tex = tex_source;
    draft.pdf_path = compiled.pdf_path;
    draft.page_count = compiled.page_count;

    Ok(GenerationResult {
        draft_id,
        pdf_path: draft.pdf_path.to_string_lossy().to_string(),
        page_count: draft.page_count,
        company: draft.company.clone(),
        role_title: draft.role_title.clone(),
        used_bullets: draft.plan.used_bullets(),
        rejected: draft.rejected.clone(),
        dropped_for_fit: 0,
        retired: Vec::new(),
    })
}

/// Renames the posting a draft is tailored to.
///
/// The company and role come from the model's read of the posting, which is a guess and is
/// often blank. What the user types here is what the library row and the exported filename
/// carry. Blank means unnamed, not an empty title. Rejects an unknown draft.
pub async fn rename(
    state: &AppState,
    draft_id: Uuid,
    company: Option<String>,
    role_title: Option<String>,
) -> Result<()> {
    let named = |v: Option<String>| v.map(|s| s.trim().to_string()).filter(|s| !s.is_empty());
    let mut drafts = state.drafts.lock().await;
    let draft = drafts
        .get_mut(&draft_id)
        .ok_or_else(|| AppError::NotFound("draft".into()))?;
    draft.company = named(company);
    draft.role_title = named(role_title);
    Ok(())
}

/// Writes a draft to the library: the application, the resume, and one provenance row
/// per printed line.
pub async fn commit(state: &AppState, draft_id: Uuid, applied: bool) -> Result<Uuid> {
    let draft = state
        .drafts
        .lock()
        .await
        .remove(&draft_id)
        .ok_or_else(|| AppError::NotFound("draft".into()))?;

    let status = if applied {
        ApplicationStatus::Applied
    } else {
        ApplicationStatus::Saved
    };
    let application = db::application::create(
        &state.pool,
        draft.url.as_deref(),
        draft.company.as_deref(),
        draft.role_title.as_deref(),
        &draft.job_text,
        draft.job_source,
        &draft.parsed,
        status,
    )
    .await?;

    let used = draft.plan.used_bullets();
    let resume = db::resume::insert(
        &state.pool,
        application.id,
        &draft.tex,
        None,
        state.llm.model(),
        PROMPT_VERSION,
        draft.feedback.as_deref(),
        draft.page_count,
        &used,
    )
    .await?;

    // The PDF moves out of the draft directory only once it belongs to a stored resume.
    let dest = state.resume_path(resume.id);
    if let Some(dir) = dest.parent() {
        tokio::fs::create_dir_all(dir).await?;
    }
    tokio::fs::copy(&draft.pdf_path, &dest).await?;
    db::resume::set_pdf_path(&state.pool, resume.id, &dest.to_string_lossy()).await?;
    let _ = tokio::fs::remove_dir_all(state.draft_dir(draft_id)).await;

    Ok(application.id)
}

/// Forgets a draft and its working directory. Writes nothing.
pub async fn discard(state: &AppState, draft_id: Uuid) -> Result<()> {
    state.drafts.lock().await.remove(&draft_id);
    let _ = tokio::fs::remove_dir_all(state.draft_dir(draft_id)).await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::skills::test_skills;

    #[test]
    fn the_keyword_scan_finds_skills_by_alias_and_ignores_the_rest() {
        let skills = test_skills(&[
            ("PostgreSQL", &["postgres"]),
            ("Python", &[]),
            ("Kubernetes", &["k8s"]),
        ]);
        let job = "You will write Python services on top of Postgres. Docker a plus.";
        let found = keyword_scan(job, &skills).hard_skills;
        assert!(found.contains(&"Python".to_string()));
        assert!(found.contains(&"PostgreSQL".to_string()));
        assert!(!found.contains(&"Kubernetes".to_string()));
    }
}
