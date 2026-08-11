//! Repository round-trips against a scratch database.
//!
//! Needs a Postgres the tests may write to:
//!     createdb resume_fixer_test
//!     TEST_DATABASE_URL=postgresql://localhost/resume_fixer_test cargo test --test db
//!
//! Without that variable every test here reports itself skipped rather than failing, so
//! `cargo test` stays green on a machine with no server running.

use chrono::NaiveDate;
use resume_fixer_lib::db;
use resume_fixer_lib::domain::*;
use sqlx::migrate::MigrateDatabase;
use sqlx::{PgPool, Postgres};
use uuid::Uuid;

async fn pool() -> Option<PgPool> {
    let url = std::env::var("TEST_DATABASE_URL").ok()?;
    let pool = db::pool::lazy(&url).expect("pool builds");
    db::pool::migrate(&pool).await.expect("migrations run");
    Some(pool)
}

macro_rules! db_test {
    ($pool:ident) => {
        match pool().await {
            Some(p) => p,
            None => {
                eprintln!("skipped: TEST_DATABASE_URL is not set");
                return;
            }
        }
    };
}

/// Builds an isolated experience → role → bullet tree and returns its ids.
async fn seed_tree(pool: &PgPool, org: &str) -> (Uuid, Uuid, Uuid) {
    let experience = db::experience::upsert(
        pool,
        &ExperienceInput {
            id: None,
            kind: ExperienceKind::Work,
            org_name: org.into(),
            location: Some("Remote".into()),
            url: None,
            tech_line: None,
            display_order: 99,
            is_active: true,
            is_pinned: false,
        },
    )
    .await
    .expect("experience inserts");

    let role = db::role::upsert(
        pool,
        &RoleInput {
            id: None,
            experience_id: experience.id,
            title: "Software Engineering Intern".into(),
            location: None,
            start_date: NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
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
        pool,
        &BulletInput {
            id: None,
            role_id: role.id,
            text: "Built a thing that measurably improved another thing by 25%.".into(),
            display_order: 0,
            is_active: true,
        },
    )
    .await
    .expect("bullet inserts");

    (experience.id, role.id, bullet.id)
}

#[tokio::test]
async fn a_full_experience_tree_round_trips() {
    let pool = db_test!(pool);
    let org = format!("Round Trip {}", Uuid::new_v4());
    let (experience_id, role_id, bullet_id) = seed_tree(&pool, &org).await;

    db::skill::set_bullet_skills(&pool, bullet_id, &["Rust".into(), "PostgreSQL".into()])
        .await
        .expect("tags save");
    db::skill::set_experience_skills(&pool, experience_id, &["Rust".into()])
        .await
        .expect("experience tags save");

    let details = db::experience::list_details(&pool)
        .await
        .expect("read back");
    let mine = details
        .iter()
        .find(|d| d.experience.id == experience_id)
        .expect("the experience comes back");

    assert_eq!(mine.experience.org_name, org);
    assert_eq!(mine.skills.len(), 1);
    assert_eq!(mine.roles.len(), 1);
    assert_eq!(mine.roles[0].role.id, role_id);
    assert_eq!(mine.roles[0].bullets.len(), 1);
    let tags: Vec<&str> = mine.roles[0].bullets[0]
        .skills
        .iter()
        .map(|s| s.slug.as_str())
        .collect();
    assert!(
        tags.contains(&"rust") && tags.contains(&"postgresql"),
        "{tags:?}"
    );

    db::experience::delete(&pool, experience_id)
        .await
        .expect("delete");
}

#[tokio::test]
async fn deleting_an_experience_cascades_to_roles_and_bullets() {
    let pool = db_test!(pool);
    let (experience_id, role_id, bullet_id) =
        seed_tree(&pool, &format!("Cascade {}", Uuid::new_v4())).await;

    db::experience::delete(&pool, experience_id)
        .await
        .expect("delete");

    assert!(db::bullet::get(&pool, bullet_id).await.is_err());
    let orphan_roles: i64 = sqlx::query_scalar("SELECT count(*) FROM roles WHERE id = $1")
        .bind(role_id)
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(orphan_roles, 0);
}

#[tokio::test]
async fn a_bullet_cited_by_a_resume_cannot_be_deleted() {
    let pool = db_test!(pool);
    let (experience_id, _, bullet_id) =
        seed_tree(&pool, &format!("Provenance {}", Uuid::new_v4())).await;

    let application = db::application::create(
        &pool,
        None,
        Some("Acme"),
        Some("Intern"),
        "a job description long enough to look real",
        JobSource::Pasted,
        &serde_json::json!({}),
        ApplicationStatus::Applied,
    )
    .await
    .expect("application inserts");

    let used = vec![UsedBullet {
        bullet_id,
        source_text: "source".into(),
        rendered_text: "rendered".into(),
        was_reworded: false,
        org_name: "Acme".into(),
    }];
    db::resume::insert(
        &pool,
        application.id,
        "\\documentclass{article}",
        None,
        "fixture",
        "test",
        None,
        1,
        &used,
    )
    .await
    .expect("resume inserts");

    // ON DELETE RESTRICT: provenance outranks tidying up the vault.
    assert!(db::bullet::delete(&pool, bullet_id).await.is_err());

    db::application::delete(&pool, application.id)
        .await
        .expect("delete application");
    db::experience::delete(&pool, experience_id)
        .await
        .expect("delete experience");
}

#[tokio::test]
async fn a_status_change_is_stamped_once_and_recorded_every_time() {
    let pool = db_test!(pool);
    let application = db::application::create(
        &pool,
        None,
        Some("History Co"),
        None,
        "another job description",
        JobSource::Pasted,
        &serde_json::json!({}),
        ApplicationStatus::Saved,
    )
    .await
    .expect("application inserts");

    db::application::set_status(&pool, application.id, ApplicationStatus::Applied)
        .await
        .unwrap();
    let after_apply = db::application::get_detail(&pool, application.id)
        .await
        .unwrap();
    let applied_at = after_apply
        .application
        .applied_at
        .expect("applied stamps a date");

    // The assessment and round statuses go through the same encode and decode as the old
    // ones; a name the database does not know fails here and nowhere else.
    for status in [
        ApplicationStatus::OaReceived,
        ApplicationStatus::OaCompleted,
        ApplicationStatus::Interview1,
        ApplicationStatus::Interview2,
        ApplicationStatus::Interview3,
    ] {
        db::application::set_status(&pool, application.id, status)
            .await
            .unwrap();
        let read = db::application::get_detail(&pool, application.id)
            .await
            .unwrap();
        assert_eq!(read.application.status, status);
    }

    db::application::set_status(&pool, application.id, ApplicationStatus::Rejected)
        .await
        .unwrap();
    let after_reject = db::application::get_detail(&pool, application.id)
        .await
        .unwrap();

    assert_eq!(after_reject.application.status, ApplicationStatus::Rejected);
    assert_eq!(
        after_reject.application.applied_at,
        Some(applied_at),
        "a later status must not move the sent date"
    );
    assert_eq!(
        after_reject.history.len(),
        8,
        "saved, applied, five rounds, rejected"
    );

    db::application::delete(&pool, application.id)
        .await
        .unwrap();
}

#[tokio::test]
async fn skills_upsert_by_slug_not_by_display_name() {
    let pool = db_test!(pool);
    let a = db::skill::upsert_by_name(&pool, "Node.js").await.unwrap();
    let b = db::skill::upsert_by_name(&pool, "node js").await.unwrap();
    assert_eq!(a.id, b.id, "the same slug must not create two rows");
    assert_eq!(a.slug, "node-js");
}

#[tokio::test]
async fn listing_a_skill_by_hand_toggles_without_losing_the_row() {
    let pool = db_test!(pool);
    // A slug nothing has seen, so the test reads the same on a fresh database as on a
    // populated one. A fixed name passes only where an earlier run happened to leave it.
    let name = format!("Kubernetes {}", Uuid::new_v4().simple());

    let tagged = db::skill::upsert_by_name(&pool, &name).await.unwrap();
    assert!(
        tagged.always_list,
        "a first-seen tag starts listed — a skill worth tagging is worth printing"
    );

    let listed = db::skill::add_listed(&pool, &name.to_lowercase())
        .await
        .unwrap();
    assert_eq!(
        listed.id, tagged.id,
        "the same slug must not create two rows"
    );
    assert_eq!(listed.name, name, "an existing row keeps its display name");
    assert!(listed.always_list);

    db::skill::set_always_list(&pool, listed.id, false)
        .await
        .unwrap();
    let after = db::skill::list(&pool).await.unwrap();
    let row = after
        .iter()
        .find(|s| s.id == listed.id)
        .expect("row survives");
    assert!(!row.always_list, "unlisting keeps the row for matching");

    // The guarantee in `upsert_by_name`: re-saving a bullet must not re-check the box.
    let retagged = db::skill::upsert_by_name(&pool, &name).await.unwrap();
    assert!(
        !retagged.always_list,
        "tagging again does not undo a box the user unchecked"
    );
}

#[tokio::test]
async fn an_active_tagged_bullet_is_retrievable_as_a_candidate() {
    let pool = db_test!(pool);
    let (_, _, bullet_id) = seed_tree(&pool, &format!("Candidate {}", Uuid::new_v4())).await;
    db::skill::set_bullet_skills(&pool, bullet_id, &["Rust".into()])
        .await
        .expect("tags save");

    let candidates = db::bullet::candidates(&pool).await.unwrap();
    let mine = candidates
        .iter()
        .find(|c| c.bullet_id == bullet_id)
        .expect("an active bullet under an active role and experience is a candidate");
    assert!(
        mine.skills.iter().any(|(slug, _)| slug == "rust"),
        "a candidate carries its slugs, which is what retrieval matches on: {:?}",
        mine.skills
    );
}

/// The branch every fresh install lands on: a server that is up, with no database yet.
///
/// Also the check that the app ships nothing personal — a database the app created for
/// itself has the schema and an empty vault, because `seed/` is not a migration.
#[tokio::test]
async fn is_ready_creates_a_database_that_does_not_exist() {
    let Ok(url) = std::env::var("TEST_DATABASE_URL") else {
        eprintln!("skipped: TEST_DATABASE_URL is not set");
        return;
    };
    // The same server under a name nothing has created. Query parameters are not carried
    // over; TEST_DATABASE_URL points at a local scratch server and does not use them.
    let (base, _) = url.rsplit_once('/').expect("the URL names a database");
    let fresh = format!("{base}/rf_autocreate_{}", Uuid::new_v4().simple());

    let pool = db::pool::lazy(&fresh).expect("pool builds");
    assert!(
        db::pool::is_ready(&pool, &fresh).await,
        "is_ready creates the database and brings the schema up"
    );
    assert!(
        db::profile::get(&pool)
            .await
            .expect("the schema is there to query")
            .is_none(),
        "a database the app created for itself holds no profile"
    );

    pool.close().await;
    Postgres::drop_database(&fresh)
        .await
        .expect("the scratch database drops");
}
