# tx

Simple speech-to-text CLI. Start talking, hit Enter, get text.

## Installation

### Homebrew

```bash
brew install bwl/ettio/tx
```

### From source

```bash
cargo install --path .
```

## Usage

```bash
tx                    # Record, transcribe, copy to clipboard
tx -q                 # Quiet mode (text only to stdout)
tx -o ~/notes         # Save to custom directory
tx --no-clip          # Skip clipboard copy
tx -q | pbcopy        # Pipe to other commands
```

## Options

```
-o, --output-dir <DIR>  Output directory [default: /tmp]
-q, --quiet             Quiet mode (text only to stdout)
    --no-clip           Skip copying to clipboard
-h, --help              Print help
-V, --version           Print version
```

## How it works

1. On first run, downloads the Whisper model (~141MB)
2. Starts recording immediately
3. Press Enter to stop recording
4. Transcribes locally using Whisper (offline, private)
5. Saves timestamped file and copies to clipboard

Set `TX_MODEL_PATH` to use a custom model location.

## License

MIT OR Apache-2.0
