//! The base resume from build to saved file, through the real service layer.
//!
//! Needs Postgres and Tectonic:
//!     TEST_DATABASE_URL=postgresql://localhost/resume_fixer_test \
//!         cargo test --test base_round_trip -- --ignored

use chrono::NaiveDate;
use resume_fixer_lib::config::Config;
use resume_fixer_lib::db;
use resume_fixer_lib::domain::*;
use resume_fixer_lib::render::templates::BUILTINS;
use resume_fixer_lib::services;
use resume_fixer_lib::state::AppState;
use uuid::Uuid;

async fn state() -> Option<AppState> {
    let url = std::env::var("TEST_DATABASE_URL").ok()?;
    let mut config = Config::default();
    config.database.url = url;
    let dir = std::env::temp_dir().join(format!("resume-fixer-base-{}", Uuid::new_v4()));
    let state = AppState::new(config, dir).expect("state builds");
    db::pool::migrate(&state.pool)
        .await
        .expect("migrations run");
    db::template::sync_builtins(&state.pool, &BUILTINS)
        .await
        .expect("templates sync");
    Some(state)
}

/// An edit applied to the preview has to reach the file that gets saved.
#[tokio::test]
#[ignore = "needs Postgres and Tectonic"]
async fn an_edited_preview_is_what_save_writes() {
    let Some(state) = state().await else {
        eprintln!("skipped: TEST_DATABASE_URL is not set");
        return;
    };

    db::profile::upsert(
        &state.pool,
        &Profile {
            full_name: "Test Person".into(),
            phone: None,
            email: Some("test@example.com".into()),
            links: vec![],
            interests: None,
        },
    )
    .await
    .expect("profile saves");

    let org = format!("Edit Round Trip {}", Uuid::new_v4());
    let experience = db::experience::upsert(
        &state.pool,
        &ExperienceInput {
            id: None,
            kind: ExperienceKind::Work,
            org_name: org.clone(),
            location: Some("Remote".into()),
            url: None,
            link_text: None,
            tech_line: None,
            display_order: 0,
            is_active: true,
            is_pinned: true,
        },
    )
    .await
    .expect("experience inserts");
    let role = db::role::upsert(
        &state.pool,
        &RoleInput {
            id: None,
            experience_id: experience.id,
            title: "Software Engineering Intern".into(),
            location: None,
            start_date: NaiveDate::from_ymd_opt(2026, 6, 1),
            end_date: None,
            date_override: None,
            gpa: None,
            display_order: 0,
            is_active: true,
        },
    )
    .await
    .expect("role inserts");
    let bullet = db::bullet::upsert(
        &state.pool,
        &BulletInput {
            id: None,
            role_id: role.id,
            text: "Built a service that cut request latency by 25%.".into(),
            display_order: 0,
            is_active: true,
        },
    )
    .await
    .expect("bullet inserts");

    let preview = services::base::render(&state, None)
        .await
        .expect("the base resume compiles");
    assert!(
        preview
            .used_bullets
            .iter()
            .any(|b| b.bullet_id == bullet.id),
        "the seeded bullet has to be on the page for the edit to mean anything"
    );

    const EDITED: &str = "Rewrote this line by hand before saving it.";
    let edits: Vec<BulletEdit> = preview
        .used_bullets
        .iter()
        .map(|b| BulletEdit {
            bullet_id: b.bullet_id,
            text: if b.bullet_id == bullet.id {
                EDITED.to_string()
            } else {
                b.rendered_text.clone()
            },
            keep: true,
        })
        .collect();

    let revised = services::base::revise(&state, edits)
        .await
        .expect("the edit recompiles");
    assert!(
        revised
            .used_bullets
            .iter()
            .any(|b| b.rendered_text == EDITED),
        "the edit did not reach the preview"
    );

    let saved = services::base::save(&state, format!("Edited {}", Uuid::new_v4()))
        .await
        .expect("save writes the resume");
    let tex: String = sqlx::query_scalar("SELECT tex_source FROM base_resumes WHERE id = $1")
        .bind(saved.id)
        .fetch_one(&state.pool)
        .await
        .expect("the row is there");
    assert!(
        tex.contains(EDITED),
        "the saved .tex is the unedited one:\n{tex}"
    );
    let pdf = saved.pdf_path.expect("a saved resume has a PDF");
    assert!(std::path::Path::new(&pdf).exists(), "{pdf} was not written");

    services::base::delete(&state, saved.id).await.unwrap();
    db::experience::delete(&state.pool, experience.id)
        .await
        .unwrap();
}
