//! Model path resolution and download for Whisper models.

use anyhow::{Context, Result, bail};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs::{self, File};
use std::io::{self, BufRead, Write};
use std::path::PathBuf;

const MODEL_NAME: &str = "ggml-base.en.bin";
const MODEL_URL: &str = "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin";
const MODEL_SIZE: u64 = 147_964_211; // ~141MB

/// Returns the path to the Whisper model, downloading if necessary.
pub fn get_model_path() -> Result<PathBuf> {
    // Check environment variable first
    if let Ok(path) = std::env::var("TX_MODEL_PATH") {
        let path = PathBuf::from(path);
        if path.exists() {
            return Ok(path);
        }
    }

    // Check standard location
    let data_dir = dirs::data_local_dir()
        .context("Cannot determine local data directory")?
        .join("tx")
        .join("models");

    let model_path = data_dir.join(MODEL_NAME);

    if model_path.exists() {
        return Ok(model_path);
    }

    // Model not found - offer to download
    first_run_wizard(&data_dir, &model_path)?;

    Ok(model_path)
}

fn first_run_wizard(data_dir: &PathBuf, model_path: &PathBuf) -> Result<()> {
    eprintln!("\n\x1b[93mFirst run setup\x1b[0m");
    eprintln!("tx needs to download the Whisper speech recognition model (~141MB).");
    eprintln!("This only happens once.\n");
    eprint!("Download now? [Y/n] ");
    io::stderr().flush()?;

    let stdin = io::stdin();
    let mut line = String::new();
    stdin.lock().read_line(&mut line)?;

    let response = line.trim().to_lowercase();
    if response == "n" || response == "no" {
        bail!(
            "Model download cancelled.\n\n\
            You can download manually:\n\n  \
            mkdir -p {}\n  \
            curl -L {} \\\n    \
            -o {}\n\n\
            Or set TX_MODEL_PATH to point to your model file.",
            data_dir.display(),
            MODEL_URL,
            model_path.display()
        );
    }

    // Create directory
    fs::create_dir_all(data_dir).context("Failed to create models directory")?;

    // Download with progress bar
    download_model(model_path)?;

    eprintln!("\n\x1b[92mModel downloaded successfully!\x1b[0m\n");

    Ok(())
}

fn download_model(model_path: &PathBuf) -> Result<()> {
    eprintln!();

    let client = reqwest::blocking::Client::new();
    let response = client
        .get(MODEL_URL)
        .send()
        .context("Failed to connect to Hugging Face")?;

    if !response.status().is_success() {
        bail!("Download failed: HTTP {}", response.status());
    }

    let total_size = response.content_length().unwrap_or(MODEL_SIZE);

    let pb = ProgressBar::new(total_size);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
            .unwrap()
            .progress_chars("#>-"),
    );

    // Download to temp file first, then rename
    let temp_path = model_path.with_extension("bin.tmp");
    let mut file = File::create(&temp_path).context("Failed to create temp file")?;

    let mut downloaded: u64 = 0;
    let mut reader = response;

    loop {
        let mut buffer = [0u8; 8192];
        match std::io::Read::read(&mut reader, &mut buffer) {
            Ok(0) => break,
            Ok(n) => {
                file.write_all(&buffer[..n])?;
                downloaded += n as u64;
                pb.set_position(downloaded);
            }
            Err(e) => {
                let _ = fs::remove_file(&temp_path);
                bail!("Download failed: {}", e);
            }
        }
    }

    pb.finish_with_message("done");

    // Rename temp file to final path
    fs::rename(&temp_path, model_path).context("Failed to finalize model file")?;

    Ok(())
}
