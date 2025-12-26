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

## Setup

tx requires a Whisper model file. Download the base English model:

```bash
mkdir -p ~/.local/share/tx/models
curl -L https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.en.bin \
  -o ~/.local/share/tx/models/ggml-base.en.bin
```

Or set `TX_MODEL_PATH` to point to your model file.

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

1. Starts recording immediately on launch
2. Press Enter to stop recording
3. Transcribes locally using Whisper (offline, private)
4. Saves timestamped file and copies to clipboard

## License

MIT OR Apache-2.0
