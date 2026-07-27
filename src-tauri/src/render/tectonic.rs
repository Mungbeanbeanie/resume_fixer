//! Tectonic invoked as an external binary.

use crate::error::{AppError, Result};
use std::path::{Path, PathBuf};

pub struct Compiled {
    pub pdf_path: PathBuf,
    pub page_count: i32,
}

/// Reads the page count out of the TeX log.
///
/// The engine prints `Output written on resume.xdv (2 pages, 51231 bytes)` — singular
/// "page" for one, and `.xdv` rather than `.pdf` under Tectonic's XeTeX. Parsing the log
/// avoids pulling in a PDF crate to count objects.
pub fn page_count(log: &str) -> Option<i32> {
    let tail = log.rsplit_once("Output written on")?.1;
    let inside = tail.split_once('(')?.1;
    let digits: String = inside.chars().take_while(char::is_ascii_digit).collect();
    digits.parse().ok()
}

/// Compiles `tex` to a PDF in `workdir` and returns its path and page count.
///
/// Rejects a non-zero exit or a missing page count in the log; both mean the document
/// did not come out the other side, and a caller must not treat that as a resume.
pub async fn compile(tectonic_path: &str, tex: &str, workdir: &Path) -> Result<Compiled> {
    tokio::fs::create_dir_all(workdir).await?;
    let tex_path = workdir.join("resume.tex");
    tokio::fs::write(&tex_path, tex).await?;

    let out = tokio::process::Command::new(tectonic_path)
        .arg("-X")
        .arg("compile")
        // The terse CLI output has no page count; the TeX log does.
        .arg("--keep-logs")
        .arg("--outdir")
        .arg(workdir)
        .arg(&tex_path)
        .output()
        .await
        .map_err(|e| AppError::Render(format!("could not run `{tectonic_path}`: {e}")))?;

    let log = format!(
        "{}{}{}",
        String::from_utf8_lossy(&out.stderr),
        String::from_utf8_lossy(&out.stdout),
        tokio::fs::read_to_string(workdir.join("resume.log"))
            .await
            .unwrap_or_default()
    );
    if !out.status.success() {
        return Err(AppError::Render(last_lines(&log, 25)));
    }
    let pdf_path = workdir.join("resume.pdf");
    if !pdf_path.exists() {
        return Err(AppError::Render(last_lines(&log, 25)));
    }
    Ok(Compiled {
        page_count: page_count(&log)
            .ok_or_else(|| AppError::Render("no page count in the TeX log".into()))?,
        pdf_path,
    })
}

fn last_lines(s: &str, n: usize) -> String {
    let lines: Vec<&str> = s.lines().collect();
    lines[lines.len().saturating_sub(n)..].join("\n")
}

/// True when the configured binary answers `--version`.
pub async fn is_available(tectonic_path: &str) -> bool {
    tokio::process::Command::new(tectonic_path)
        .arg("--version")
        .output()
        .await
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::page_count;

    #[test]
    fn reads_the_count_from_a_real_log_tail() {
        assert_eq!(
            page_count("note: Writing `resume.pdf` (51231 bytes)\nOutput written on resume.pdf (1 page, 51231 bytes).\n"),
            Some(1)
        );
        assert_eq!(
            page_count("Output written on resume.pdf (12 pages, 9 bytes)."),
            Some(12)
        );
        assert_eq!(page_count("! LaTeX Error: something broke"), None);
    }
}
