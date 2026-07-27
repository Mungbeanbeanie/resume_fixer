//! The one-page loop: render, count pages, give something up, render again.

use super::plan::ResumePlan;
use crate::error::Result;
use crate::render::{tectonic, tex};
use std::path::{Path, PathBuf};

/// Six passes is enough to shed a third of a full page. Past that the plan is not too
/// long, it is wrong, and dropping more bullets makes a worse resume rather than a
/// shorter one.
const MAX_PASSES: usize = 6;

pub struct Fitted {
    pub tex: String,
    pub pdf_path: PathBuf,
    pub page_count: i32,
    pub dropped: i32,
}

/// Compiles the plan, shrinking it until it fits one page.
///
/// Rejects nothing: if the plan still overflows after `MAX_PASSES`, the last successful
/// compile is returned with its real page count so the caller can warn instead of failing.
pub async fn fit_to_one_page(
    tectonic_path: &str,
    template: &str,
    plan: &mut ResumePlan,
    workdir: &Path,
) -> Result<Fitted> {
    let mut dropped = 0;
    loop {
        let source = tex::render(template, &plan.to_render_input())?;
        let compiled = tectonic::compile(tectonic_path, &source, workdir).await?;
        let fits = compiled.page_count <= 1;
        let out_of_passes = dropped as usize >= MAX_PASSES;

        if fits || out_of_passes || plan.drop_lowest().is_none() {
            return Ok(Fitted {
                tex: source,
                pdf_path: compiled.pdf_path,
                page_count: compiled.page_count,
                dropped,
            });
        }
        dropped += 1;
        tracing::info!(
            "resume ran to {} pages, dropped bullet {dropped}",
            compiled.page_count
        );
    }
}
