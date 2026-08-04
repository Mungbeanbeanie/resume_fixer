//! LaTeX escaping, date formatting, and the Tera context builder.
//!
//! Escaping happens here and nowhere else. Templates receive strings that are already
//! safe, so a template edit cannot introduce an injection and a call site cannot forget.

use crate::domain::Profile;
use crate::error::{AppError, Result};
use chrono::{Datelike, NaiveDate};
use serde::Serialize;
use tera::{Context, Tera};

/// Escapes a plain-text string for LaTeX body text.
///
/// Rejects nothing — every input is representable. The backslash is replaced first so the
/// backslashes introduced by later replacements are not re-escaped.
pub fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '\\' => out.push_str("\\textbackslash{}"),
            '&' => out.push_str("\\&"),
            '%' => out.push_str("\\%"),
            '$' => out.push_str("\\$"),
            '#' => out.push_str("\\#"),
            '_' => out.push_str("\\_"),
            '{' => out.push_str("\\{"),
            '}' => out.push_str("\\}"),
            '~' => out.push_str("\\textasciitilde{}"),
            '^' => out.push_str("\\textasciicircum{}"),
            _ => out.push(c),
        }
    }
    out
}

/// Escapes a URL for use inside `\href{...}`.
///
/// `\href` takes its argument verbatim apart from the characters TeX itself eats, so only
/// those are escaped; percent-encoded octets must survive unchanged.
pub fn escape_url(s: &str) -> String {
    s.replace('%', "\\%")
        .replace('#', "\\#")
        .replace('&', "\\&")
        .replace('_', "\\_")
}

fn month(m: u32) -> &'static str {
    // Short months keep their full name; the rest are abbreviated with a period. This is
    // the convention the reference resume already uses.
    [
        "Jan.", "Feb.", "Mar.", "Apr.", "May", "June", "July", "Aug.", "Sept.", "Oct.", "Nov.",
        "Dec.",
    ][(m - 1) as usize]
}

fn stamp(d: NaiveDate) -> String {
    format!("{} {}", month(d.month()), d.year())
}

/// `Aug. 2025 -- May 2029`, `June 2026 -- Present`, or a single stamp when a role starts
/// and ends in the same month. `date_override` wins outright when the user set one.
pub fn format_dates(
    start: NaiveDate,
    end: Option<NaiveDate>,
    date_override: Option<&str>,
) -> String {
    if let Some(o) = date_override.map(str::trim).filter(|o| !o.is_empty()) {
        return o.to_string();
    }
    match end {
        None => format!("{} -- Present", stamp(start)),
        Some(e) if e.year() == start.year() && e.month() == start.month() => stamp(start),
        Some(e) => format!("{} -- {}", stamp(start), stamp(e)),
    }
}

// ── Context shape ───────────────────────────────────────────────────────
// Plain text in, escaped on the way into Tera.

#[derive(Debug, Clone, Default)]
pub struct RoleBlock {
    pub title: String,
    pub dates: String,
    pub bullets: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ExperienceBlock {
    pub org: String,
    pub location: String,
    pub roles: Vec<RoleBlock>,
}

#[derive(Debug, Clone, Default)]
pub struct ProjectBlock {
    pub name: String,
    pub tech: String,
    pub dates: String,
    pub bullets: Vec<String>,
}

/// Everything the template prints, in plain text.
#[derive(Debug, Clone, Default)]
pub struct RenderInput {
    pub profile: Option<Profile>,
    pub education: Vec<ExperienceBlock>,
    pub skills_line: String,
    pub experience: Vec<ExperienceBlock>,
    pub projects: Vec<ProjectBlock>,
    /// Printed only by templates that name `activities`; see `ResumePlan::prune_for`.
    pub activities: Vec<ExperienceBlock>,
    pub certifications: Vec<String>,
}

#[derive(Serialize)]
struct LinkOut {
    label: String,
    url: String,
}

#[derive(Serialize)]
struct RoleOut {
    title: String,
    dates: String,
    bullets: Vec<String>,
}

#[derive(Serialize)]
struct ExperienceOut {
    org: String,
    location: String,
    roles: Vec<RoleOut>,
}

#[derive(Serialize)]
struct ProjectOut {
    name: String,
    tech: String,
    dates: String,
    bullets: Vec<String>,
}

fn escape_block(b: &ExperienceBlock) -> ExperienceOut {
    ExperienceOut {
        org: escape(&b.org),
        location: escape(&b.location),
        roles: b
            .roles
            .iter()
            .map(|r| RoleOut {
                title: escape(&r.title),
                dates: escape(&r.dates),
                bullets: r.bullets.iter().map(|t| escape(t)).collect(),
            })
            .collect(),
    }
}

/// Builds the Tera context, escaping every user-supplied string on the way in.
pub fn build_context(input: &RenderInput) -> Context {
    let mut ctx = Context::new();
    if let Some(p) = &input.profile {
        ctx.insert("full_name", &escape(&p.full_name));
        ctx.insert("phone", &p.phone.as_deref().map(escape).unwrap_or_default());
        ctx.insert("email", &p.email.as_deref().map(escape).unwrap_or_default());
        ctx.insert(
            "links",
            &p.links
                .iter()
                .map(|l| LinkOut {
                    label: escape(&l.label),
                    url: escape_url(&l.url),
                })
                .collect::<Vec<_>>(),
        );
    } else {
        ctx.insert("full_name", "");
        ctx.insert("phone", "");
        ctx.insert("email", "");
        ctx.insert("links", &Vec::<LinkOut>::new());
    }
    ctx.insert(
        "education",
        &input.education.iter().map(escape_block).collect::<Vec<_>>(),
    );
    ctx.insert("skills_line", &escape(&input.skills_line));
    ctx.insert(
        "experience",
        &input
            .experience
            .iter()
            .map(escape_block)
            .collect::<Vec<_>>(),
    );
    ctx.insert(
        "projects",
        &input
            .projects
            .iter()
            .map(|p| ProjectOut {
                name: escape(&p.name),
                tech: escape(&p.tech),
                dates: escape(&p.dates),
                bullets: p.bullets.iter().map(|t| escape(t)).collect(),
            })
            .collect::<Vec<_>>(),
    );
    ctx.insert(
        "activities",
        &input
            .activities
            .iter()
            .map(escape_block)
            .collect::<Vec<_>>(),
    );
    ctx.insert(
        "certifications",
        &input
            .certifications
            .iter()
            .map(|c| escape(c))
            .collect::<Vec<_>>(),
    );
    ctx
}

/// Renders a `.tex` document. Autoescaping is off — HTML escaping would corrupt LaTeX,
/// and the context is already escaped by `build_context`.
pub fn render(template: &str, input: &RenderInput) -> Result<String> {
    Tera::one_off(template, &build_context(input), false)
        .map_err(|e| AppError::Render(format!("template error: {e}")))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::render::templates::{DEFAULT_TEMPLATE, SIMPLIFY_TEMPLATE};

    #[test]
    fn every_special_character_is_escaped() {
        assert_eq!(escape("a & b"), "a \\& b");
        assert_eq!(escape("100%"), "100\\%");
        assert_eq!(escape("$5"), "\\$5");
        assert_eq!(escape("C#"), "C\\#");
        assert_eq!(escape("a_b"), "a\\_b");
        assert_eq!(escape("{x}"), "\\{x\\}");
        assert_eq!(escape("~/x"), "\\textasciitilde{}/x");
        assert_eq!(escape("2^10"), "2\\textasciicircum{}10");
        assert_eq!(escape("\\LaTeX"), "\\textbackslash{}LaTeX");
    }

    #[test]
    fn escaping_is_not_applied_twice() {
        // The backslashes introduced by escaping must not themselves be escaped.
        assert_eq!(escape("&%$"), "\\&\\%\\$");
        assert_eq!(escape("a\\&b"), "a\\textbackslash{}\\&b");
    }

    #[test]
    fn output_has_no_unescaped_specials() {
        let adversarial = "R&D 50% of $1_000 {#hash} ~tilde^ \\newcommand{\\x}{y}";
        // The commands escaping emits carry their own empty braces; everything else that
        // survives must be backslash-escaped.
        let out = escape(adversarial)
            .replace("\\textbackslash{}", "")
            .replace("\\textasciitilde{}", "")
            .replace("\\textasciicircum{}", "");
        for (i, c) in out.char_indices() {
            if "&%$#_{}".contains(c) {
                assert_eq!(&out[..i].chars().last(), &Some('\\'), "bare {c} in {out}");
            }
        }
    }

    #[test]
    fn dates_follow_the_reference_conventions() {
        let d = |y, m, day| NaiveDate::from_ymd_opt(y, m, day).unwrap();
        assert_eq!(
            format_dates(d(2025, 8, 1), Some(d(2029, 5, 1)), None),
            "Aug. 2025 -- May 2029"
        );
        assert_eq!(
            format_dates(d(2026, 6, 1), None, None),
            "June 2026 -- Present"
        );
        assert_eq!(
            format_dates(d(2026, 4, 1), Some(d(2026, 4, 30)), None),
            "Apr. 2026"
        );
        assert_eq!(
            format_dates(d(2026, 4, 1), Some(d(2026, 4, 30)), Some("Summer 2025")),
            "Summer 2025"
        );
        assert_eq!(
            format_dates(d(2026, 4, 1), None, Some("  ")),
            "Apr. 2026 -- Present"
        );
    }

    #[test]
    fn empty_sections_emit_no_headers() {
        for template in [DEFAULT_TEMPLATE, SIMPLIFY_TEMPLATE] {
            let out = render(template, &RenderInput::default()).unwrap();
            assert!(!out.contains("\\section{"), "bare section header in {out}");
            assert!(out.contains("\\end{document}"));
        }
    }

    /// The whole activities design rests on this: the same input prints an Activities
    /// section under Simplify and nothing at all under Jake's.
    #[test]
    fn activities_print_only_where_the_template_has_a_section() {
        let input = RenderInput {
            activities: vec![ExperienceBlock {
                org: "Intramural Ice Hockey".into(),
                location: "Charlottesville, VA".into(),
                roles: vec![RoleBlock {
                    title: "Team Captain".into(),
                    dates: "Jan. 2026 -- Present".into(),
                    bullets: vec![],
                }],
            }],
            ..Default::default()
        };
        let simplify = render(SIMPLIFY_TEMPLATE, &input).unwrap();
        assert!(simplify.contains("\\section{Activities \\& Leadership}"));
        assert!(simplify.contains("Intramural Ice Hockey"));

        let jake = render(DEFAULT_TEMPLATE, &input).unwrap();
        assert!(!jake.contains("Intramural Ice Hockey"));
    }

    #[test]
    fn a_bullet_with_specials_survives_into_the_document() {
        let input = RenderInput {
            experience: vec![ExperienceBlock {
                org: "R&D Corp".into(),
                location: "Malvern, PA".into(),
                roles: vec![RoleBlock {
                    title: "Intern".into(),
                    dates: "June 2026 -- Aug. 2026".into(),
                    bullets: vec!["Cut cost 50% using C# & Node_js".into()],
                }],
            }],
            ..Default::default()
        };
        let out = render(DEFAULT_TEMPLATE, &input).unwrap();
        assert!(out.contains("{R\\&D Corp}{Malvern, PA}"));
        assert!(out.contains("Cut cost 50\\% using C\\# \\& Node\\_js"));
    }
}
