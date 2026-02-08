# tx

Speech-to-text CLI. Records audio, transcribes locally via Whisper, copies result to clipboard.

## Architecture

Flat `src/` layout, 6 modules:

- `main.rs` — CLI parsing (clap derive), subcommand dispatch, orchestrates record flow
- `audio.rs` — Mic capture via cpal. Records at device native rate, resamples to 16kHz mono
- `model.rs` — Resolves whisper model path (`TX_MODEL_PATH` env or `~/.local/share/tx/models/`). Auto-downloads `ggml-base.en.bin` on first run
- `transcribe.rs` — Whisper inference via whisper-rs (beam search, English only)
- `output.rs` — File save (timestamped to output dir) and clipboard copy (arboard)
- `db.rs` — SQLite history at `~/.local/share/tx/history.db`. Short hash IDs, prefix-match lookup

## Key details

- Rust 2024 edition
- macOS only (release CI builds aarch64 + x86_64 darwin)
- Distributed via Homebrew (`brew install bwl/ettio/tx`)
- No tests currently
- whisper-rs links whisper.cpp natively — builds take a while

## Commands

```
tx                  # default: record -> transcribe -> save -> clipboard
tx -q               # quiet mode: text only to stdout, status on stderr
tx last             # print most recent transcript
tx history          # list recent transcripts (alias: tx log)
tx show <id>        # print transcript by ID prefix
tx copy <id>        # copy transcript to clipboard by ID prefix
```

## Build

```
cargo build --release
```

First build compiles whisper.cpp via whisper-rs-sys (needs cmake). Subsequent builds are fast.
