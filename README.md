# sayit

## Overview

This utility converts text to speech using the OpenAI API. It supports various input methods, audio formats, and voices, allowing for flexible and customizable text-to-speech conversion.

## Features

- Accepts input from a file, clipboard, or stdin
- Supports multiple audio formats: Opus, AAC, FLAC, PCM, MP3
- Provides various voices: Alloy, Echo, Fable, Onyx, Nova, Shimmer
- Adjustable speech speed (0.25 - 4.0)
- High Definition (HD) audio option
- Outputs to a file or plays audio directly

## Dependencies

Ensure you have the following dependencies installed:

```toml
[dependencies]
bytes = "1.0"
clap = { version = "4.0", features = ["derive"] }
clipboard = "0.6"
env_logger = "0.10"
log = "0.4"
reqwest = { version = "0.11", features = ["json"] }
rodio = "0.14"
tokio = { version = "1.0", features = ["full"] }
```

## Usage

Run the utility using the following command:

```sh
cargo run -- [OPTIONS]
```

### Options

- `-i, --input-file <FILE>`: Specify the input file to read from
- `-o, --output-file <FILE>`: Specify the output file to write audio to
- `-f, --format <FORMAT>`: Set the audio format (Opus, AAC, FLAC, PCM, MP3)
- `-v, --voice <VOICE>`: Choose the voice (Alloy, Echo, Fable, Onyx, Nova, Shimmer)
- `-s, --speed <SPEED>`: Set the speech speed (0.25 - 4.0)
- `--hd`: Enable High Definition audio
- `-c, --clipboard`: Use the clipboard as input
- `-d, --use-stdin`: Read input from stdin

### Environment Variables

- `OPENAI_API_KEY`: Set your OpenAI API key

## Examples

### Convert Text from a File

```sh
cargo run -- --input-file input.txt --output-file output.mp3 --voice nova --format mp3 --speed 1.0
```

### Convert Text from Clipboard

```sh
cargo run -- --clipboard --output-file output.flac --voice echo --format flac
```

### Convert Text from Stdin

```sh
echo "Hello, world!" | cargo run -- --use-stdin --output-file output.opus --voice shimmer --format opus
```

## Code Explanation

The main components of the code include:

1. **Command-line Interface (CLI) Parsing**:
    - Uses `clap` to define and parse command-line arguments.
    - Defines options for input file, output file, format, voice, speed, HD flag, clipboard flag, and stdin flag.

2. **Enum Definitions**:
    - `ResponseFormat` and `Voice` enums define the supported audio formats and voices.

3. **Helper Functions**:
    - `split_input`: Splits the input text into manageable chunks for processing.

4. **Asynchronous Functions**:
    - `fetch_and_process_audio`: Fetches and processes audio data from the OpenAI API.
    - `play_audio_from_queue`: Plays audio directly from the queue.
    - `audio_to_output_file`: Writes audio data to the specified output file.

5. **Main Function**:
    - Initializes the logger.
    - Parses CLI arguments and determines the input source.
    - Splits the input text into chunks and processes each chunk asynchronously.
    - Either plays the audio or writes it to an output file based on the provided options.

