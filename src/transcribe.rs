//! Whisper transcription via whisper-rs.

use anyhow::{Context, Result};
use std::path::Path;
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

/// Transcribes audio samples using Whisper.
///
/// Audio should be f32 samples at 16kHz mono.
pub fn transcribe(audio: &[f32], model_path: &Path, quiet: bool) -> Result<String> {
    if !quiet {
        eprintln!("\x1b[90m(Loading model...)\x1b[0m");
    }

    let ctx = WhisperContext::new_with_params(
        model_path.to_str().context("Invalid model path")?,
        WhisperContextParameters::default(),
    )
    .context("Failed to load Whisper model")?;

    let mut state = ctx.create_state().context("Failed to create Whisper state")?;

    let mut params = FullParams::new(SamplingStrategy::BeamSearch { beam_size: 5, patience: -1.0 });
    params.set_language(Some("en"));
    params.set_print_special(false);
    params.set_print_progress(false);
    params.set_print_realtime(false);
    params.set_print_timestamps(false);

    state
        .full(params, audio)
        .context("Failed to transcribe audio")?;

    let num_segments = state.full_n_segments();

    let mut text = String::new();
    for i in 0..num_segments {
        if let Some(segment) = state.get_segment(i) {
            if let Ok(segment_text) = segment.to_str_lossy() {
                text.push_str(&segment_text);
                text.push(' ');
            }
        }
    }

    Ok(text.trim().to_string())
}
