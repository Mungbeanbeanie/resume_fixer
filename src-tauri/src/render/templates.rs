//! The templates shipped with the app.
//!
//! These are re-synced into the `templates` table on every startup, so editing a
//! `.tex.tera` file is what changes a built-in — never an UPDATE.

/// Jake's Resume, compiled from the user's reference `.tex`.
pub const DEFAULT_TEMPLATE: &str = include_str!("../../../templates/resume.tex.tera");

/// The Simplify layout. The only built-in that prints an Activities section.
pub const SIMPLIFY_TEMPLATE: &str = include_str!("../../../templates/simplify.tex.tera");

/// `(name, source)`. The first entry is the one made active on a fresh database.
pub const BUILTINS: [(&str, &str); 2] = [
    ("Jake's Resume", DEFAULT_TEMPLATE),
    ("Simplify", SIMPLIFY_TEMPLATE),
];
