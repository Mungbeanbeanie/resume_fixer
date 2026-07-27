//! The whole pipeline against the real local stack.
//!
//! Needs Postgres, Ollama with the configured model, and Tectonic, so it is ignored by
//! default:
//!     TEST_DATABASE_URL=postgresql://localhost/resume_fixer_test \
//!         cargo test --test generate_end_to_end -- --ignored --nocapture

use resume_fixer_lib::config::Config;
use resume_fixer_lib::domain::JobSource;
use resume_fixer_lib::services::generate;
use resume_fixer_lib::state::AppState;

const POSTING: &str = "\
Software Engineering Intern, Data Platform

We are building the ingestion layer for a medical device telemetry product. You will write
Python services that move sensor data into PostgreSQL, containerize them with Docker, and
deploy on AWS. You will work cross-functionally with hardware engineers.

Responsibilities
- Build and maintain Python data pipelines
- Model relational schemas in PostgreSQL
- Package services with Docker and ship them to AWS

Qualifications
- Currently pursuing a degree in computer science
- Experience with Python, SQL, and Linux
- Familiarity with REST APIs and CI/CD is a plus
";

#[tokio::test]
#[ignore = "needs Postgres, Ollama, and Tectonic"]
async fn a_posting_becomes_a_one_page_pdf_with_provenance() {
    // Surfaces the "fell back to …" warnings, which is how a silently degraded run shows up.
    let _ = tracing_subscriber::fmt().with_test_writer().try_init();

    let Ok(url) = std::env::var("TEST_DATABASE_URL") else {
        eprintln!("skipped: TEST_DATABASE_URL is not set");
        return;
    };
    let mut config = Config::load().expect("config loads");
    config.database.url = url;

    let data_dir = std::env::temp_dir().join("resume-fixer-e2e");
    let state = AppState::new(config, data_dir).expect("state builds");
    resume_fixer_lib::db::pool::migrate(&state.pool)
        .await
        .expect("migrations run");

    let started = std::time::Instant::now();
    let result = generate::generate(&state, POSTING, None, JobSource::Pasted, None)
        .await
        .expect("generation succeeds");

    println!(
        "{} bullets, {} reworded, {} rejected, {} dropped, {} page(s) in {:?}",
        result.used_bullets.len(),
        result
            .used_bullets
            .iter()
            .filter(|b| b.was_reworded)
            .count(),
        result.rejected.len(),
        result.dropped_for_fit,
        result.page_count,
        started.elapsed()
    );
    for r in &result.rejected {
        println!("  rejected: {} — {}", r.reason, r.attempted);
    }
    println!("  pdf: {}", result.pdf_path);

    assert_eq!(result.page_count, 1, "the fit loop must land on one page");
    assert!(!result.used_bullets.is_empty(), "something must be printed");
    assert!(
        std::path::Path::new(&result.pdf_path).exists(),
        "the PDF is on disk"
    );

    // Nothing printed may be text the model wrote out of nothing: every line either matches
    // its stored bullet or is a grounded rewrite of it.
    let candidates = resume_fixer_lib::db::bullet::candidates(&state.pool)
        .await
        .unwrap();
    for used in &result.used_bullets {
        let stored = candidates
            .iter()
            .find(|c| c.bullet_id == used.bullet_id)
            .expect("every printed line traces to a stored bullet");
        if !used.was_reworded {
            assert_eq!(used.rendered_text, stored.text);
        }
    }

    generate::discard(&state, result.draft_id).await.unwrap();
}
