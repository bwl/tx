//! Audio recording via cpal.

use anyhow::{Context, Result};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use std::io::{self, BufRead};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};

pub const SAMPLE_RATE: u32 = 16000;

/// Records audio until Enter is pressed.
/// Returns f32 samples at 16kHz mono.
pub fn record_until_enter(quiet: bool) -> Result<Vec<f32>> {
    let host = cpal::default_host();
    let device = host
        .default_input_device()
        .context("No audio input device available")?;

    let supported_config = device
        .supported_input_configs()
        .context("Failed to get supported input configs")?
        .find(|c| c.channels() == 1 && c.min_sample_rate().0 <= SAMPLE_RATE && c.max_sample_rate().0 >= SAMPLE_RATE)
        .or_else(|| {
            // Fall back to any config and we'll convert
            device.supported_input_configs().ok()?.next()
        })
        .context("No suitable audio config found")?;

    let sample_format = supported_config.sample_format();
    let config: cpal::StreamConfig = supported_config.with_sample_rate(cpal::SampleRate(SAMPLE_RATE)).into();

    let samples: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
    let stop_flag = Arc::new(AtomicBool::new(false));

    let samples_clone = Arc::clone(&samples);
    let stop_clone = Arc::clone(&stop_flag);

    let err_fn = |err| eprintln!("Audio stream error: {}", err);

    let stream = match sample_format {
        cpal::SampleFormat::F32 => {
            device.build_input_stream(
                &config,
                move |data: &[f32], _: &_| {
                    if !stop_clone.load(Ordering::Relaxed) {
                        let mut samples = samples_clone.lock().unwrap();
                        samples.extend_from_slice(data);
                    }
                },
                err_fn,
                None,
            )?
        }
        cpal::SampleFormat::I16 => {
            device.build_input_stream(
                &config,
                move |data: &[i16], _: &_| {
                    if !stop_clone.load(Ordering::Relaxed) {
                        let mut samples = samples_clone.lock().unwrap();
                        samples.extend(data.iter().map(|&s| s as f32 / 32768.0));
                    }
                },
                err_fn,
                None,
            )?
        }
        cpal::SampleFormat::I32 => {
            device.build_input_stream(
                &config,
                move |data: &[i32], _: &_| {
                    if !stop_clone.load(Ordering::Relaxed) {
                        let mut samples = samples_clone.lock().unwrap();
                        samples.extend(data.iter().map(|&s| s as f32 / 2147483648.0));
                    }
                },
                err_fn,
                None,
            )?
        }
        _ => anyhow::bail!("Unsupported sample format: {:?}", sample_format),
    };

    if !quiet {
        eprintln!("\x1b[93m[Recording...]\x1b[0m Press ENTER when done.");
    }

    stream.play().context("Failed to start audio stream")?;

    // Wait for Enter
    let stdin = io::stdin();
    let _ = stdin.lock().lines().next();

    stop_flag.store(true, Ordering::Relaxed);
    drop(stream);

    let samples = Arc::try_unwrap(samples)
        .map_err(|_| anyhow::anyhow!("Failed to unwrap samples"))?
        .into_inner()
        .unwrap();

    Ok(samples)
}
