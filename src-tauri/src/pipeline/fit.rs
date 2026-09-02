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

/// Compiles the plan, shrinking it until it fits one page, then growing it back into
/// whatever room is left.
///
/// Shrinks in two moves, in this order: retire the lowest-scoring experience whole, and
/// only when nothing may be retired, thin the biggest section's weakest bullet. Depth beats
/// breadth — three entries with three lines each carry more than eight with one — so the
/// page is bought by cutting the weakest evidence entirely rather than starving every entry.
///
/// Growing runs the same two moves backwards, breadth first: another experience says more
/// than a third line on one that is already printed. A page that stops halfway down is a
/// page of evidence thrown away, and the bullet cap deliberately starts lean so this pass
/// has something to spend.
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

    let compile = |plan: &ResumePlan| {
        let source = tex::render(template, &plan.to_render_input())?;
        Ok::<_, crate::error::AppError>(source)
    };

    let fitted = loop {
        let source = compile(plan)?;
        let compiled = tectonic::compile(tectonic_path, &source, workdir).await?;

        if compiled.page_count <= 1 || passes >= MAX_PASSES {
            break (source, compiled);
        }

        if let Some(org) = plan.retire_weakest() {
            tracing::info!("resume ran to {} pages, retired {org}", compiled.page_count);
        } else if plan.drop_lowest().is_some() {
            dropped += 1;
            tracing::info!(
                "resume ran to {} pages, dropped bullet {dropped}",
                compiled.page_count
            );
        } else {
            break (source, compiled);
        }
        passes += 1;
    };

    let (mut source, mut compiled) = fitted;

    // Nothing to grow into if it never reached one page.
    while compiled.page_count <= 1 && passes < MAX_PASSES {
        let snapshot = plan.clone();
        if !plan.restore_section() && !plan.restore_bullet() {
            break;
        }
        passes += 1;

        let grown_source = compile(plan)?;
        let grown = tectonic::compile(tectonic_path, &grown_source, workdir).await?;
        if grown.page_count > 1 {
            // It did not fit. Go back, and recompile because that attempt overwrote the
            // PDF this function is about to hand back.
            *plan = snapshot;
            source = compile(plan)?;
            compiled = tectonic::compile(tectonic_path, &source, workdir).await?;
            break;
        }
        source = grown_source;
        compiled = grown;
    }

    Ok(Fitted {
        tex: source,
        pdf_path: compiled.pdf_path,
        page_count: compiled.page_count,
        dropped,
        // What is still retired, not what was retired along the way: the grow pass may have
        // put one back, and the UI must not name an experience that is on the page.
        retired: plan.retired_names(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pipeline::plan::{PlanBullet, PlanRole, PlanSection};
    use crate::render::templates::DEFAULT_TEMPLATE;
    use uuid::Uuid;

    fn section(org: &str, score: f32, bullets: usize) -> PlanSection {
        let line = "Designed Java-based RESTful services supporting high-volume banking \
                    transactions, improving request latency by 25%";
        PlanSection {
            experience_id: Uuid::new_v4(),
            org: org.into(),
            location: "Charlottesville, VA".into(),
            tech: String::new(),
            url: None,
            link_text: None,
            keep_empty: false,
            score,
            pinned: false,
            roles: vec![PlanRole {
                role_id: Uuid::new_v4(),
                title: "Intern".into(),
                dates: "June 2026 -- Aug. 2026".into(),
                bullets: (0..bullets)
                    .map(|i| PlanBullet {
                        bullet_id: Uuid::new_v4(),
                        source_text: line.into(),
                        text: format!("{line} ({i})"),
                        was_reworded: false,
                        // Falls away fast, so a project's third bullet is well under the
                        // ratio that would let it print without the grow pass.
                        score: (1.0 - i as f32 * 0.3).max(0.05),
                    })
                    .collect(),
            }],
        }
    }

    /// The loop has to fill the page, not just avoid overflowing it. A short resume leaves
    /// the bottom third white, and every benched line is evidence that went unspent.
    ///
    /// Needs Tectonic on PATH:
    ///     cargo test --lib fit -- --ignored --nocapture
    #[tokio::test]
    #[ignore = "needs tectonic on PATH"]
    async fn the_grow_pass_spends_the_space_left_at_the_bottom() {
        let mut plan = ResumePlan {
            experience: vec![section("Rajant Health", 2.0, 5)],
            projects: vec![section("Car Simulation", 1.0, 3)],
            ..Default::default()
        };
        plan.cap_bullets();
        let after_cap = plan.bullet_count();
        assert_eq!(after_cap, 5, "three for the job, two for the project");

        let dir = std::env::temp_dir().join("resume-fixer-grow");
        let fitted = fit_to_one_page("tectonic", DEFAULT_TEMPLATE, &mut plan, &dir)
            .await
            .expect("the document compiles");

        println!(
            "grow: {after_cap} bullets after the cap, {} after fitting, {} page(s)",
            plan.bullet_count(),
            fitted.page_count
        );
        assert_eq!(fitted.page_count, 1, "it must still come out one page");
        assert!(
            plan.bullet_count() > after_cap,
            "the space at the bottom went unspent: still {after_cap} bullets"
        );
    }

    /// A resume that already fills the page must come back untouched, not one line longer.
    #[tokio::test]
    #[ignore = "needs tectonic on PATH"]
    async fn a_full_page_is_left_alone() {
        let mut plan = ResumePlan {
            experience: (0..8)
                .map(|i| section(&format!("Employer {i}"), 2.0, 4))
                .collect(),
            ..Default::default()
        };
        plan.cap_bullets();

        let dir = std::env::temp_dir().join("resume-fixer-full");
        let fitted = fit_to_one_page("tectonic", DEFAULT_TEMPLATE, &mut plan, &dir)
            .await
            .expect("the document compiles");

        println!(
            "full: {} bullets, {} page(s), retired {:?}",
            plan.bullet_count(),
            fitted.page_count,
            fitted.retired
        );
        assert_eq!(fitted.page_count, 1);
        assert_eq!(
            fitted.retired,
            plan.retired_names(),
            "the report names what is still off the page"
        );
    }
}
