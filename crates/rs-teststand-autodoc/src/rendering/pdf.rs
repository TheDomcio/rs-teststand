//! Headless PDF rendering.
//!
//! Prints a standalone HTML document with a browser that is already installed,
//! invoked directly on the command line. No driver process, no listening port,
//! nothing fetched: the page is a local file and the browser is a local
//! executable.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use crate::error::Error;

/// Browsers able to print a page, in the order they are tried.
///
/// Chromium's `--print-to-pdf` is the mechanism; Edge ships with Windows, so it
/// is looked for first and Chrome is the fallback.
const BROWSERS: [&str; 4] = [
    r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
    r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
    r"C:\Program Files\Google\Chrome\Application\chrome.exe",
    r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
];

/// The first installed browser, if any.
fn find_browser() -> Option<PathBuf> {
    BROWSERS
        .iter()
        .map(PathBuf::from)
        .find(|path| path.is_file())
}

/// A `file:` URL for a local path.
///
/// Built by hand rather than by string concatenation because the drive letter
/// matters: `file://c/...` drops the colon and the browser reports the page as
/// missing, then prints its own error page instead of the document.
fn file_url(path: &Path) -> String {
    let text = path.display().to_string();
    format!("file:///{}", text.replace(char::from(92), "/"))
}

/// Renders an HTML string to a PDF file at `output_pdf_path`.
///
/// # Errors
/// [`Error`] when no browser is installed, the browser cannot be run, or it
/// produces no readable PDF.
pub fn html_to_pdf(html_content: &str, output_pdf_path: &Path) -> Result<(), Error> {
    let Some(browser) = find_browser() else {
        return Err(Error::Io(std::io::Error::other(
            "no browser found to print with: install Microsoft Edge or Google Chrome",
        )));
    };

    let temp_dir = std::env::temp_dir().join("rs_teststand_autodoc");
    fs::create_dir_all(&temp_dir)?;
    let temp_html_path = temp_dir.join(format!("report_{}.html", std::process::id()));
    fs::write(&temp_html_path, html_content)?;

    if let Some(parent) = output_pdf_path.parent() {
        fs::create_dir_all(parent)?;
    }
    let absolute_pdf_path = if output_pdf_path.is_absolute() {
        output_pdf_path.to_path_buf()
    } else {
        std::env::current_dir()?.join(output_pdf_path)
    };

    let status = Command::new(&browser)
        .arg("--headless")
        .arg("--disable-gpu")
        .arg("--no-sandbox")
        // Suppresses the date and file path the browser otherwise stamps on
        // every page. The older `--print-to-pdf-no-header` is accepted and
        // silently ignored by current builds.
        .arg("--no-pdf-header-footer")
        .arg(format!("--print-to-pdf={}", absolute_pdf_path.display()))
        .arg(file_url(&temp_html_path))
        .status();

    let _ = fs::remove_file(&temp_html_path);

    let status = status.map_err(|error| {
        Error::Io(std::io::Error::other(format!(
            "could not run {}: {error}",
            browser.display()
        )))
    })?;
    if !status.success() {
        return Err(Error::Io(std::io::Error::other(format!(
            "{} exited with {status} without printing",
            browser.display()
        ))));
    }

    // The browser reports success even when it printed its own error page, so
    // the result is checked rather than assumed.
    let written = fs::metadata(&absolute_pdf_path).map_or(0, |meta| meta.len());
    if written == 0 {
        return Err(Error::Io(std::io::Error::other(
            "the browser produced no PDF",
        )));
    }
    let header = fs::read(&absolute_pdf_path)?;
    if !header.starts_with(b"%PDF") {
        return Err(Error::Io(std::io::Error::other(
            "the printed file is not a PDF",
        )));
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{file_url, find_browser, html_to_pdf};
    use std::path::{Path, PathBuf};

    #[test]
    fn a_windows_path_keeps_its_drive_letter() {
        // `file://c/...` without the colon is a different, missing URL, and the
        // browser prints its own error page for it.
        let url = file_url(Path::new(r"C:\docs\report.html"));
        assert_eq!(url, "file:///C:/docs/report.html");
        assert!(!url.contains(char::from(92)));
    }

    #[test]
    #[ignore = "prints with a real browser"]
    fn renders_html_to_a_readable_pdf() -> Result<(), Box<dyn std::error::Error>> {
        if find_browser().is_none() {
            println!("  skipped: no browser installed");
            return Ok(());
        }
        let target: PathBuf = std::env::temp_dir()
            .join("rs_teststand_autodoc_pdf_test")
            .join("output.pdf");

        html_to_pdf(
            "<!doctype html><html><head><title>T</title></head><body><h1>Hello</h1></body></html>",
            &target,
        )?;

        let bytes = std::fs::read(&target)?;
        assert!(bytes.starts_with(b"%PDF"), "not a PDF");
        assert!(bytes.len() > 1000, "suspiciously small: {}", bytes.len());
        let _ = std::fs::remove_file(&target);
        Ok(())
    }
}
