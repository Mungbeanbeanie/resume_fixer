//! The one-page loop: render, count pages, give something up, render again.

use super::plan::ResumePlan;
use crate::error::Result;
use crate::render::{tectonic, tex};
use std::path::{Path, PathBuf};

/// A pass now retires a whole experience before it thins any bullets, so the ceiling has
/// to clear the number of entries a full vault can carry — six was sized for bullet drops
/// alone and left a 14-entry vault stranded at two pages. Each pass costs one Tectonic
/// compile, roughly 0.7s warm, so twelve is a few seconds spent to reach one page.
const MAX_PASSES: usize = 12;

pub struct Fitted {
    pub tex: String,
    pub pdf_path: PathBuf,
    pub page_count: i32,
    pub dropped: i32,
    /// Experiences retired whole, by org name, so the UI can say which and not just how many.
    pub retired: Vec<String>,
}

/// Compiles the plan, shrinking it until it fits one page.
///
/// Shrinks in two moves, in this order: retire the lowest-scoring experience whole, and
/// only when nothing may be retired, thin the biggest section's weakest bullet. Depth beats
/// breadth — three entries with three lines each carry more than eight with one — so the
/// page is bought by cutting the weakest evidence entirely rather than starving every entry.
///
/// Rejects nothing: if the plan still overflows after `MAX_PASSES`, the last successful
/// compile is returned with its real page count so the caller can warn instead of failing.
pub async fn fit_to_one_page(
    tectonic_path: &str,
    template: &str,
    plan: &mut ResumePlan,
    workdir: &Path,
) -> Result<Fitted> {
    let mut passes = 0;
    let mut dropped = 0;
    let mut retired = Vec::new();
    loop {
        let source = tex::render(template, &plan.to_render_input())?;
        let compiled = tectonic::compile(tectonic_path, &source, workdir).await?;

        if compiled.page_count <= 1 || passes >= MAX_PASSES {
            return Ok(Fitted {
                tex: source,
                pdf_path: compiled.pdf_path,
                page_count: compiled.page_count,
                dropped,
                retired,
            });
        }

        if let Some(org) = plan.retire_weakest() {
            tracing::info!("resume ran to {} pages, retired {org}", compiled.page_count);
            retired.push(org);
        } else if plan.drop_lowest().is_some() {
            dropped += 1;
            tracing::info!(
                "resume ran to {} pages, dropped bullet {dropped}",
                compiled.page_count
            );
        } else {
            return Ok(Fitted {
                tex: source,
                pdf_path: compiled.pdf_path,
                page_count: compiled.page_count,
                dropped,
                retired,
            });
        }
        passes += 1;
    }
}
