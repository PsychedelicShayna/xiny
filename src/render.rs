use std::path::PathBuf;
use std::process;

use anyhow::{self as ah, Context};

const FALLBACK_VIEWERS: &[&str] = &["glow", "mdt", "bat", "less"];

fn viewer_in_path(name: &str) -> bool {
    std::env::var("PATH")
        .unwrap_or_default()
        .split(':')
        .any(|dir| std::path::Path::new(dir).join(name).exists())
}

/// Outputs the Markdown document using the preferred renderer, falling back
pub fn print_document(path: &PathBuf, preferred: Option<&str>) -> ah::Result<()> {
    if !path.exists() {
        return Err(ah::anyhow!("Document does not exist: {}", path.display()));
    }

    let renderer = preferred
        .filter(|r| !r.is_empty() && viewer_in_path(r))
        .or_else(|| FALLBACK_VIEWERS.iter().find(|&&r| viewer_in_path(r)).copied());

    match renderer {
        Some(r) => {
            process::Command::new(r)
                .arg(path.display().to_string())
                .status()
                .context("print_document spawning renderer")?;
        }
        None => {
            let document =
                std::fs::read_to_string(path).context("print_document reading document")?;
            print!("{}", document);
        }
    }

    Ok(())
}
