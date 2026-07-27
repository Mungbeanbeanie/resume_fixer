//! Vault operations that need more than one repository.

use crate::db;
use crate::domain::Suggestion;
use crate::error::Result;
use crate::llm;
use crate::pipeline::{grounding, skills::SkillIndex};
use crate::state::AppState;
use uuid::Uuid;

/// Asks the model for alternate phrasings of one stored bullet.
///
/// Rejects any suggestion that fails grounding — the Improve button must not be a way
/// around the gate the generation pipeline enforces. Surviving suggestions are stored as
/// unapproved variants so accepting one later is a single click.
pub async fn suggest_improvements(state: &AppState, bullet_id: Uuid) -> Result<Vec<Suggestion>> {
    let bullet = db::bullet::get(&state.pool, bullet_id).await?;
    let all_skills = db::skill::list(&state.pool).await?;
    let index = SkillIndex::new(&all_skills);

    let tagged: Vec<String> = db::experience::list_details(&state.pool)
        .await?
        .iter()
        .flat_map(|e| e.roles.iter())
        .flat_map(|r| r.bullets.iter())
        .find(|b| b.bullet.id == bullet_id)
        .map(|b| b.skills.iter().map(|s| s.slug.clone()).collect())
        .unwrap_or_default();

    let named = tagged
        .iter()
        .filter_map(|s| index.display(s))
        .collect::<Vec<_>>()
        .join(", ");

    let improvements = llm::improve_bullet(state.llm.as_ref(), &bullet.text, &named).await?;

    let mut out = Vec::new();
    for text in improvements.suggestions {
        let text = text.trim();
        if text == bullet.text {
            continue;
        }
        if let Err(reason) = grounding::check(&bullet.text, text, &tagged, &index) {
            tracing::info!("dropped a suggestion for {bullet_id}: {reason}");
            continue;
        }
        let variant = db::bullet::add_variant(&state.pool, bullet_id, text, "ai_suggested").await?;
        out.push(Suggestion {
            variant_id: variant.id,
            text: variant.text,
        });
    }
    Ok(out)
}
