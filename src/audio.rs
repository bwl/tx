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

    // Get the default config - most reliable
    let default_config = device
        .default_input_config()
        .context("Failed to get default input config")?;

    let device_sample_rate = default_config.sample_rate().0;
    let channels = default_config.channels() as usize;
    let sample_format = default_config.sample_format();

    let config: cpal::StreamConfig = default_config.into();

    let samples: Arc<Mutex<Vec<f32>>> = Arc::new(Mutex::new(Vec::new()));
    let stop_flag = Arc::new(AtomicBool::new(false));

    let samples_clone = Arc::clone(&samples);
    let stop_clone = Arc::clone(&stop_flag);

    let err_fn = |err| eprintln!("Audio stream error: {}", err);

    // Capture at device's native rate and channels
    let stream = match sample_format {
        cpal::SampleFormat::F32 => {
            device.build_input_stream(
                &config,
                move |data: &[f32], _: &_| {
                    if !stop_clone.load(Ordering::Relaxed) {
                        let mut samples = samples_clone.lock().unwrap();
                        // Convert to mono if stereo
                        if channels == 2 {
                            samples.extend(data.chunks(2).map(|c| (c[0] + c[1]) / 2.0));
                        } else {
                            samples.extend_from_slice(data);
                        }
                    }
                },
                err_fn,
                None,
            )?
        }
        cpal::SampleFormat::I16 => {
            let samples_clone = Arc::clone(&samples);
            let stop_clone = Arc::clone(&stop_flag);
            device.build_input_stream(
                &config,
                move |data: &[i16], _: &_| {
                    if !stop_clone.load(Ordering::Relaxed) {
                        let mut samples = samples_clone.lock().unwrap();
                        if channels == 2 {
                            samples.extend(data.chunks(2).map(|c| {
                                (c[0] as f32 / 32768.0 + c[1] as f32 / 32768.0) / 2.0
                            }));
                        } else {
                            samples.extend(data.iter().map(|&s| s as f32 / 32768.0));
                        }
                    }
                },
                err_fn,
                None,
            )?
        }
        cpal::SampleFormat::I32 => {
            let samples_clone = Arc::clone(&samples);
            let stop_clone = Arc::clone(&stop_flag);
            device.build_input_stream(
                &config,
                move |data: &[i32], _: &_| {
                    if !stop_clone.load(Ordering::Relaxed) {
                        let mut samples = samples_clone.lock().unwrap();
                        if channels == 2 {
                            samples.extend(data.chunks(2).map(|c| {
                                (c[0] as f32 / 2147483648.0 + c[1] as f32 / 2147483648.0) / 2.0
                            }));
                        } else {
                            samples.extend(data.iter().map(|&s| s as f32 / 2147483648.0));
                        }
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

    // Resample to 16kHz if needed
    if device_sample_rate != SAMPLE_RATE {
        Ok(resample(&samples, device_sample_rate, SAMPLE_RATE))
    } else {
        Ok(samples)
    }
}

/// Simple linear resampling
fn resample(samples: &[f32], from_rate: u32, to_rate: u32) -> Vec<f32> {
    let ratio = from_rate as f64 / to_rate as f64;
    let new_len = (samples.len() as f64 / ratio) as usize;
    let mut output = Vec::with_capacity(new_len);

    for i in 0..new_len {
        let src_idx = i as f64 * ratio;
        let idx = src_idx as usize;
        let frac = src_idx - idx as f64;

        let sample = if idx + 1 < samples.len() {
            samples[idx] * (1.0 - frac as f32) + samples[idx + 1] * frac as f32
        } else if idx < samples.len() {
            samples[idx]
        } else {
            0.0
        };
        output.push(sample);
    }

    output
}
