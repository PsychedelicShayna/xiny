use std::path::PathBuf;
use std::process;

use std::io::BufRead;

use anyhow::{self as ah, Context};

/// Outputs the Markdown document to stdout using the provided renderer, or as
/// raw plaintext if None is provided.
pub fn print_document(path: &PathBuf, renderer: Option<&str>) -> ah::Result<()> {
    if !path.exists() {
        return Err(ah::anyhow!("The document does not exist."));
    }

    // Simplest case: print the document as plaintext.
    if renderer.is_none() {
        let document = std::fs::read_to_string(path).context("print_document reading document")?;
        println!("{}", document);
        return Ok(());
    }

    // If a renderer is provided, we need to check if the binary exists.
    let renderer = renderer.unwrap();

    let env_path =
        std::env::var("PATH").context("print_document getting PATH environment variable")?;

    let env_paths = env_path.split(':');

    let mut binary_exists: bool = false;

    for path in env_paths {
        let path = std::path::Path::new(path).join(renderer);

        if path.exists() {
            binary_exists = true;
            break;
        }
    }

    if !binary_exists {
        return Err(ah::anyhow!(
            "Could not find the renderer binary in the PATH environment variable."
        ));
    }

    process::Command::new(renderer)
        .arg(path.display().to_string())
        .status()
        .context("print_document checking renderer binary")?;

    Ok(())
}
