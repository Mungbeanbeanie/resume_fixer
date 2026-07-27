//! Turning a job posting page into plain text.

use crate::error::{AppError, Result};
use scraper::{ElementRef, Html, Node};

/// Chrome and navigation never contain the posting.
const STRIPPED: [&str; 9] = [
    "script", "style", "nav", "header", "footer", "svg", "noscript", "form", "iframe",
];

/// Elements that end a line of prose.
const BLOCK: [&str; 15] = [
    "p", "div", "li", "br", "tr", "section", "article", "h1", "h2", "h3", "h4", "h5", "h6", "ul",
    "ol",
];

/// A page is only a job posting if it says so. These stems appear in every real posting
/// and in almost no login wall or cookie banner.
const REQUIRED_TOKENS: [&str; 5] = [
    "responsibilit",
    "qualificat",
    "requirement",
    "experience",
    "skills",
];

/// Text of a subtree with the stripped tags removed, plus the number of elements it spans.
///
/// The element count is the denominator of the density score, so markup-heavy navigation
/// scores low even when it holds a lot of words.
fn text_and_tags(root: ElementRef) -> (String, usize) {
    let mut out = String::new();
    let mut tags = 0usize;
    let mut stack = vec![*root];
    while let Some(node) = stack.pop() {
        match node.value() {
            Node::Element(e) => {
                if STRIPPED.contains(&e.name()) {
                    continue;
                }
                tags += 1;
                if BLOCK.contains(&e.name()) {
                    out.push('\n');
                }
            }
            Node::Text(t) => out.push_str(t),
            _ => {}
        }
        // Reverse so children come off the stack in document order.
        for child in node.children().collect::<Vec<_>>().into_iter().rev() {
            stack.push(child);
        }
    }
    (out, tags.max(1))
}

/// Collapses runs of whitespace, keeping at most one blank line between blocks.
pub fn normalize(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    for line in raw.lines() {
        let line = line.split_whitespace().collect::<Vec<_>>().join(" ");
        if line.is_empty() {
            if !out.ends_with("\n\n") && !out.is_empty() {
                out.push('\n');
            }
        } else {
            out.push_str(&line);
            out.push('\n');
        }
    }
    out.trim().to_string()
}

/// True when the text reads like a job posting rather than a wall or an error page.
pub fn looks_like_a_posting(text: &str, min_chars: usize) -> bool {
    if text.len() < min_chars {
        return false;
    }
    let lower = text.to_lowercase();
    REQUIRED_TOKENS.iter().any(|t| lower.contains(t))
}

/// Extracts the densest block of prose from an HTML page.
///
/// Rejects a page whose best candidate is shorter than `min_chars` or reads like chrome
/// rather than a posting — the caller then offers the paste box instead of guessing.
pub fn extract(html: &str, min_chars: usize) -> Result<String> {
    let doc = Html::parse_document(html);
    let mut best: Option<(f32, String)> = None;
    let mut longest: Option<(usize, String)> = None;

    for element in doc.tree.root().descendants().filter_map(ElementRef::wrap) {
        if !matches!(
            element.value().name(),
            "body" | "main" | "article" | "section" | "div" | "td"
        ) {
            continue;
        }
        let (raw, tags) = text_and_tags(element);
        let text = normalize(&raw);
        if text.is_empty() {
            continue;
        }
        // Density picks prose over navigation; the length floor keeps it from picking a
        // single dense sentence out of a long posting.
        let density = text.len() as f32 / tags as f32;
        if text.len() >= min_chars && best.as_ref().is_none_or(|(d, _)| density > *d) {
            best = Some((density, text.clone()));
        }
        if longest.as_ref().is_none_or(|(l, _)| text.len() > *l) {
            longest = Some((text.len(), text));
        }
    }

    let text = best
        .map(|(_, t)| t)
        .or_else(|| longest.map(|(_, t)| t))
        .ok_or(AppError::ExtractFailed)?;

    if !looks_like_a_posting(&text, min_chars) {
        return Err(AppError::ExtractFailed);
    }
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    const POSTING: &str = r#"
    <html><head><style>.a{color:red}</style><script>var x = 1;</script></head>
    <body>
      <nav><a href="/">Home</a><a href="/jobs">Jobs</a><a href="/about">About</a></nav>
      <header><h1>Acme Corp</h1></header>
      <div id="content"><article>
        <h2>Software Engineering Intern</h2>
        <p>Acme is hiring an intern to build data pipelines for our medical devices group.</p>
        <h3>Responsibilities</h3>
        <ul><li>Write Python services backed by PostgreSQL.</li>
            <li>Work cross-functionally with hardware engineers on sensor ingest.</li></ul>
        <h3>Qualifications</h3>
        <ul><li>Experience with Python, SQL, and Linux.</li>
            <li>Currently pursuing a degree in computer science or a related field.</li>
            <li>Familiarity with Docker and AWS is a plus.</li></ul>
      </article></div>
      <footer><a href="/privacy">Privacy</a><a href="/terms">Terms</a></footer>
    </body></html>"#;

    #[test]
    fn pulls_the_posting_and_drops_the_chrome() {
        let text = extract(POSTING, 200).unwrap();
        assert!(text.contains("Write Python services backed by PostgreSQL."));
        assert!(text.contains("Qualifications"));
        assert!(!text.contains("var x = 1"));
        assert!(!text.contains("Privacy"));
        assert!(!text.contains("color:red"));
    }

    #[test]
    fn each_list_item_stays_on_its_own_line() {
        let text = extract(POSTING, 200).unwrap();
        assert!(text
            .lines()
            .any(|l| l == "Experience with Python, SQL, and Linux."));
    }

    #[test]
    fn a_login_wall_is_rejected() {
        let wall = "<html><body><div><p>Sign in to view this job.</p></div></body></html>";
        assert!(extract(wall, 400).is_err());
    }

    #[test]
    fn a_long_page_without_posting_vocabulary_is_rejected() {
        let filler = "lorem ipsum dolor sit amet ".repeat(40);
        let page = format!("<html><body><div><p>{filler}</p></div></body></html>");
        assert!(extract(&page, 400).is_err());
    }

    #[test]
    fn whitespace_collapses_without_swallowing_paragraph_breaks() {
        assert_eq!(normalize("  a   b \n\n\n c  "), "a b\n\nc");
    }
}
