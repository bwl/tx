//! Model path resolution for Whisper models.

use anyhow::{Context, Result, bail};
use std::path::PathBuf;

const MODEL_NAME: &str = "ggml-base.en.bin";

/// Returns the path to the Whisper model.
///
/// Checks in order:
/// 1. TX_MODEL_PATH environment variable
/// 2. ~/.local/share/tx/models/ggml-base.en.bin
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

    bail!(
        "Whisper model not found.\n\n\
        Download the model with:\n\n  \
        mkdir -p ~/.local/share/tx/models\n  \
        curl -L https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin \\\n    \
        -o ~/.local/share/tx/models/ggml-base.en.bin\n\n\
        Or set TX_MODEL_PATH to point to your model file."
    );
}
