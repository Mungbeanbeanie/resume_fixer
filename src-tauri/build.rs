fn main() {
    // `sqlx::migrate!` embeds the files present when the macro expands, and tracks only
    // those. A migration added afterwards leaves the crate looking fresh, so cargo serves a
    // cached binary that silently runs an older schema. Watching the directory catches it.
    println!("cargo:rerun-if-changed=../migrations");
    tauri_build::build()
}
