//! Output handling: file save and clipboard.

use anyhow::{Context, Result};
use arboard::Clipboard;
use chrono::Local;
use std::fs;
use std::path::{Path, PathBuf};

/// Saves transcription text to a timestamped file.
///
/// Returns the path to the saved file.
pub fn save_to_file(text: &str, output_dir: &Path) -> Result<PathBuf> {
    fs::create_dir_all(output_dir).context("Failed to create output directory")?;

    let timestamp = Local::now().format("%Y-%m-%d-%H%M%S");
    let filename = format!("tx-{}.txt", timestamp);
    let path = output_dir.join(filename);

    fs::write(&path, text).context("Failed to write transcription file")?;

    Ok(path)
}

/// Copies text to the system clipboard.
pub fn copy_to_clipboard(text: &str) -> Result<()> {
    let mut clipboard = Clipboard::new().context("Failed to access clipboard")?;
    clipboard
        .set_text(text)
        .context("Failed to copy to clipboard")?;
    Ok(())
}
