//! The base resume: the whole vault through a template, with no job posting involved.
//!
//! No model runs here, so no bullet is reworded and grounding has nothing to check — the
//! text printed is the text stored. There is no fit loop either: a master resume that
//! silently dropped bullets to reach one page would be the wrong document.

use crate::db;
use crate::domain::*;
use crate::error::{AppError, Result};
use crate::pipeline::select;
use crate::render::{tectonic, tex};
use crate::state::{AppState, BaseDraft};
use uuid::Uuid;

/// The skills actually tagged somewhere in the vault, in `skills` table order.
///
/// The whole `skills` table would print rows the user never used; the vault's own tags are
/// the honest list.
fn vault_skills(vault: &[ExperienceDetail], all: &[Skill]) -> Vec<String> {
    let tagged: Vec<&str> = vault
        .iter()
        .flat_map(|e| {
            e.skills.iter().chain(
                e.roles
                    .iter()
                    .flat_map(|r| r.bullets.iter())
                    .flat_map(|b| b.skills.iter()),
            )
        })
        .map(|s| s.slug.as_str())
        .collect();
    all.iter()
        .filter(|s| tagged.contains(&s.slug.as_str()))
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

    let mut plan = select::everything(&vault, profile, vault_skills(&vault, &all_skills));
    plan.prune_for(&template.source);
    if plan.bullet_count() == 0 {
        return Err(AppError::Invalid(
            "the vault is empty — add an experience before building a base resume".into(),
        ));
    }

    let tex_source = tex::render(&template.source, &plan.to_render_input())?;
    let compiled = tectonic::compile(
        &state.config.render.tectonic_path,
        &tex_source,
        &state.base_preview_dir(),
    )
    .await?;

    let preview = BasePreview {
        template_name: template.name.clone(),
        pdf_path: compiled.pdf_path.to_string_lossy().to_string(),
        page_count: compiled.page_count,
        bullet_count: plan.bullet_count() as i32,
    };
    *state.base_preview.lock().await = Some(BaseDraft {
        template_name: template.name,
        tex: tex_source,
        pdf_path: compiled.pdf_path,
        page_count: compiled.page_count,
        bullet_count: preview.bullet_count,
    });
    Ok(preview)
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
    fn the_skills_line_holds_only_what_the_vault_tags() {
        let all = test_skills(&[("Python", &[]), ("Rust", &[]), ("SQL", &[])]);
        let vault = vec![ExperienceDetail {
            experience: Experience {
                id: Uuid::new_v4(),
                kind: ExperienceKind::Work,
                org_name: "Rajant Health".into(),
                location: None,
                url: None,
                tech_line: None,
                display_order: 0,
                is_active: true,
            },
            skills: vec![all[2].clone()],
            roles: vec![],
        }];
        assert_eq!(vault_skills(&vault, &all), vec!["SQL".to_string()]);
    }
}
