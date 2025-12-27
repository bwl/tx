//! tx - Simple speech-to-text CLI
//!
//! Start talking, hit Enter, get text.

mod audio;
mod db;
mod model;
mod output;
mod transcribe;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::io::{self, Write};
use std::path::PathBuf;
use std::process;

#[derive(Parser)]
#[command(name = "tx")]
#[command(about = "Speech-to-text CLI - start talking, hit Enter, get text")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,

    /// Output directory for transcription files
    #[arg(short, long, default_value = "/tmp", global = true)]
    output_dir: PathBuf,

    /// Quiet mode (text only to stdout)
    #[arg(short, long, global = true)]
    quiet: bool,

    /// Skip copying to clipboard
    #[arg(long, global = true)]
    no_clip: bool,
}

#[derive(Subcommand)]
enum Command {
    /// Show transcript history
    #[command(alias = "log")]
    History {
        /// Number of entries to show
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// Show a transcript by ID
    Show {
        /// Transcript ID (or prefix)
        id: String,
    },

    /// Copy a transcript to clipboard
    Copy {
        /// Transcript ID (or prefix)
        id: String,
    },
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {:#}", e);
        process::exit(1);
    }
}

fn run() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        None => record(&cli),
        Some(Command::History { limit }) => history(limit),
        Some(Command::Show { id }) => show(&id),
        Some(Command::Copy { id }) => copy(&id),
    }
}

fn record(cli: &Cli) -> Result<()> {
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

    // Save to database
    let conn = db::open()?;
    let cwd = std::env::current_dir()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|_| "unknown".to_string());
    let id = db::save(&conn, &text, &cwd)?;

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
        eprintln!("\n\x1b[92mSaved:\x1b[0m {} \x1b[90m({})\x1b[0m", out_path.display(), id);
        println!("\n---\n{}\n---\n", text);
        if !cli.no_clip {
            eprintln!("\x1b[90mCopied to clipboard.\x1b[0m");
        }
    }

    Ok(())
}

fn history(limit: usize) -> Result<()> {
    let conn = db::open()?;
    let transcripts = db::list(&conn, limit)?;

    if transcripts.is_empty() {
        println!("No transcripts yet.");
        return Ok(());
    }

    for t in transcripts {
        let preview: String = t.text.chars().take(60).collect();
        let preview = if t.text.len() > 60 {
            format!("{}...", preview)
        } else {
            preview
        };
        let time = t.timestamp.format("%Y-%m-%d %H:%M");
        println!(
            "\x1b[93m{}\x1b[0m  \x1b[90m{}\x1b[0m  {}",
            t.id, time, preview
        );
    }

    Ok(())
}

fn show(id: &str) -> Result<()> {
    let conn = db::open()?;

    match db::find_by_prefix(&conn, id)? {
        Some(t) => {
            println!("{}", t.text);
        }
        None => {
            eprintln!("No transcript found with ID starting with '{}'", id);
            process::exit(1);
        }
    }

    Ok(())
}

fn copy(id: &str) -> Result<()> {
    let conn = db::open()?;

    match db::find_by_prefix(&conn, id)? {
        Some(t) => {
            output::copy_to_clipboard(&t.text)?;
            eprintln!("Copied to clipboard.");
        }
        None => {
            eprintln!("No transcript found with ID starting with '{}'", id);
            process::exit(1);
        }
    }

    Ok(())
}
