//! tx - Simple speech-to-text CLI
//!
//! Start talking, hit Enter, get text.

mod audio;
mod model;
mod output;
mod transcribe;

use anyhow::Result;
use clap::Parser;
use std::io::{self, Write};
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "tx")]
#[command(about = "Speech-to-text CLI - start talking, hit Enter, get text")]
#[command(version)]
struct Cli {
    /// Output directory for transcription files
    #[arg(short, long, default_value = "/tmp")]
    output_dir: PathBuf,

    /// Quiet mode (text only to stdout)
    #[arg(short, long)]
    quiet: bool,

    /// Skip copying to clipboard
    #[arg(long)]
    no_clip: bool,
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {:#}", e);
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    // Get model path first (fails early with helpful message)
    let model_path = model::get_model_path()?;

    // Record audio
    let samples = audio::record_until_enter(cli.quiet)?;

    // Check for minimum audio
    if samples.len() < (audio::SAMPLE_RATE / 2) as usize {
        eprintln!("No audio recorded.");
        process::exit(1);
    }

    // Show transcribing status in quiet mode
    if cli.quiet {
        eprint!("\x1b[90mTranscribing...\x1b[0m");
        io::stderr().flush().ok();
    }

    // Transcribe
    let text = transcribe::transcribe(&samples, &model_path, cli.quiet)?;

    // Clear status line in quiet mode
    if cli.quiet {
        eprint!("\r\x1b[K");
        io::stderr().flush().ok();
    }

    if text.is_empty() {
        eprintln!("Could not transcribe.");
        process::exit(1);
    }

    // Save to file
    let out_path = output::save_to_file(&text, &cli.output_dir)?;

    // Copy to clipboard
    if !cli.no_clip {
        if let Err(e) = output::copy_to_clipboard(&text) {
            if !cli.quiet {
                eprintln!("\x1b[90m(Clipboard unavailable: {})\x1b[0m", e);
            }
        }
    }

    // Output
    if cli.quiet {
        println!("{}", text);
    } else {
        eprintln!("\n\x1b[92mSaved:\x1b[0m {}", out_path.display());
        println!("\n---\n{}\n---\n", text);
        if !cli.no_clip {
            eprintln!("\x1b[90mCopied to clipboard.\x1b[0m");
        }
    }

    Ok(())
}
