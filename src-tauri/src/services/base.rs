//! The base resume: the whole vault through a template, with no job posting involved.
//!
//! No model runs here, so no bullet is reworded and grounding has nothing to check — the
//! text printed is the text stored. The fit loop does run, ranking by intrinsic merit
//! rather than fit for a posting, so what it gives up first is the weakest evidence rather
//! than whatever happened to overflow the page.

use crate::db;
use crate::domain::*;
use crate::error::{AppError, Result};
use crate::pipeline::{fit, select};
use crate::render::{tectonic, tex};
use crate::state::{AppState, BaseDraft};
use uuid::Uuid;

/// The skills marked to print, in the order they were checked (`db::skill::list`).
///
/// The whole table would print rows the user never used, so the checkbox in the Skills
/// section decides. Tagging a new skill turns it on (`db::skill::upsert_by_name`), which
/// is why this does not read the vault's tags a second time — that would override the
/// user's own unchecking.
fn vault_skills(all: &[Skill]) -> Vec<String> {
    all.iter()
        .filter(|s| s.always_list)
        .map(|s| s.name.clone())
        .collect()
}

/// Renders every active vault entry with `template_id`, or the active template.
///
/// Rejects an empty vault: a resume with no bullets is a header and a page of white.
pub async fn render(state: &AppState, template_id: Option<Uuid>) -> Result<BasePreview> {
    let template = match template_id {
        Some(id) => db::template::get(&state.pool, id).await?,
        None => db::template::get_active(&state.pool).await?,
    }
    .ok_or_else(|| AppError::NotFound("template".into()))?;

    let vault = db::experience::list_details(&state.pool).await?;
    let profile = db::profile::get(&state.pool).await?;
    let all_skills = db::skill::list(&state.pool).await?;

    let mut plan = select::everything(&vault, profile, vault_skills(&all_skills));
    plan.prune_for(&template.source);
    plan.cap_bullets();
    if plan.bullet_count() == 0 {
        return Err(AppError::Invalid(
            "the vault is empty — add an experience before building a base resume".into(),
        ));
    }

    let fitted = fit::fit_to_one_page(
        &state.config.render.tectonic_path,
        &template.source,
        &mut plan,
        &state.base_preview_dir(),
    )
    .await?;

    let preview = BasePreview {
        template_name: template.name.clone(),
        pdf_path: fitted.pdf_path.to_string_lossy().to_string(),
        page_count: fitted.page_count,
        bullet_count: plan.bullet_count() as i32,
        used_bullets: plan.used_bullets(),
        retired: fitted.retired,
    };
    *state.base_preview.lock().await = Some(BaseDraft {
        template_id: template.id,
        template_name: template.name,
        plan,
        tex: fitted.tex,
        pdf_path: fitted.pdf_path,
        page_count: fitted.page_count,
        bullet_count: preview.bullet_count,
    });
    Ok(preview)
}

/// Reworks the previewed base resume to the user's own edits and recompiles it.
///
/// Same rule as a generated draft: the edit belongs to this document, not to the vault, and
/// no model runs. Rejects an edit set that would leave nothing to print.
pub async fn revise(state: &AppState, edits: Vec<BulletEdit>) -> Result<BasePreview> {
    // An edit may name a bullet this preview never printed; the vault is where its text lives.
    let vault = db::experience::list_details(&state.pool).await?;

    let mut held = state.base_preview.lock().await;
    let draft = held
        .as_mut()
        .ok_or_else(|| AppError::NotFound("base resume preview".into()))?;

    let template = db::template::get(&state.pool, draft.template_id)
        .await?
        .ok_or_else(|| AppError::NotFound("template".into()))?;

    draft.plan.apply_edits(&vault, &edits);
    if draft.plan.bullet_count() == 0 {
        return Err(AppError::Invalid(
            "that would leave the resume with no bullets — keep at least one".into(),
        ));
    }

    let tex_source = tex::render(&template.source, &draft.plan.to_render_input())?;
    let compiled = tectonic::compile(
        &state.config.render.tectonic_path,
        &tex_source,
        &state.base_preview_dir(),
    )
    .await?;

    draft.tex = tex_source;
    draft.pdf_path = compiled.pdf_path;
    draft.page_count = compiled.page_count;
    draft.bullet_count = draft.plan.bullet_count() as i32;

    Ok(BasePreview {
        template_name: draft.template_name.clone(),
        pdf_path: draft.pdf_path.to_string_lossy().to_string(),
        page_count: draft.page_count,
        bullet_count: draft.bullet_count,
        used_bullets: draft.plan.used_bullets(),
        retired: Vec::new(),
    })
}

/// Names the previewed resume and keeps it. Rejects saving before a preview exists —
/// there would be no PDF to copy.
pub async fn save(state: &AppState, name: String) -> Result<BaseResume> {
    let draft = state.base_preview.lock().await;
    let draft = draft
        .as_ref()
        .ok_or_else(|| AppError::NotFound("base resume preview".into()))?;

    let saved = db::base_resume::insert(
        &state.pool,
        &name,
        &draft.template_name,
        &draft.tex,
        draft.page_count,
    )
    .await?;

    let dest = state.base_path(saved.id);
    if let Some(dir) = dest.parent() {
        tokio::fs::create_dir_all(dir).await?;
    }
    tokio::fs::copy(&draft.pdf_path, &dest).await?;
    let path = dest.to_string_lossy().to_string();
    db::base_resume::set_pdf_path(&state.pool, saved.id, &path).await?;

    Ok(BaseResume {
        pdf_path: Some(path),
        ..saved
    })
}

/// Forgets a saved base resume and its PDF.
pub async fn delete(state: &AppState, id: Uuid) -> Result<()> {
    db::base_resume::delete(&state.pool, id).await?;
    let _ = tokio::fs::remove_file(state.base_path(id)).await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::skills::test_skills;

    #[test]
    fn the_skills_line_holds_only_what_is_checked() {
        let mut all = test_skills(&[("Python", &[]), ("Rust", &[]), ("SQL", &[])]);
        all[0].always_list = true;
        all[2].always_list = true;
        // Rust is unchecked, so it stays off the page even though the vault knows it.
        assert_eq!(
            vault_skills(&all),
            vec!["Python".to_string(), "SQL".to_string()]
        );
    }
}
