use bytes::Bytes;
use clap::{CommandFactory, Parser, ValueEnum};
use clipboard::{ClipboardContext, ClipboardProvider};
use env_logger;
use log;
use reqwest::Client;
use rodio::{source::Source, Decoder, OutputStream, Sink};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, Cursor, Read, Write};
use tokio::sync::mpsc;

#[derive(Parser)]
#[command(version = "0.1", about = "Text to speech utility", long_about = None)]
struct Cli {
    // optional input file to operate on
    input_file: Option<String>,

    // Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    output_file: Option<String>,

    // Set a format option
    #[arg(short = 'f', long, value_name = "FORMAT")]
    format: Option<ResponseFormat>,

    // Set a voice
    #[arg(short = 'v', long, value_name = "VOICE")]
    voice: Option<Voice>,

    // Set a speed (0.25 - 4.0)
    #[arg(short = 's', long)]
    speed: Option<f32>,

    // Boolean HD flag
    #[arg(long)]
    hd: bool,

    // Boolean clipboard flag (reads whatever is currently pasted in the clipboard.)
    #[arg(short = 'c', long)]
    clipboard: bool,

    // Read from stdin
    #[arg(short = 'd', long)]
    use_stdin: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum ResponseFormat {
    Opus,
    Aac,
    Flac,
    Pcm,
    Mp3,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
enum Voice {
    Alloy,
    Echo,
    Fable,
    Onyx,
    Nova,
    Shimmer,
}

// Helper to split input text into manageable chunks
fn split_input(input_text: &str, max_length: usize) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current_chunk = Vec::new();
    let mut current_length = 0;

    for word in input_text.split_whitespace() {
        if current_length + word.len() + 1 > max_length {
            chunks.push(current_chunk.join(" "));
            current_chunk = vec![word.to_string()];
            current_length = word.len();
        } else {
            current_chunk.push(word.to_string());
            current_length += word.len() + 1; // +1 for space
        }
    }

    if !current_chunk.is_empty() {
        chunks.push(current_chunk.join(" "));
    }

    chunks
}

async fn fetch_and_process_audio(
    text: &str,
    index: usize,
    client: &Client,
    audio_tx: mpsc::Sender<(usize, Bytes)>,
    audio_format: &str,
    reading_voice: &str,
    tts_model: &str,
    speed: f32,
) {
    log::info!("Fetching audio for chunk {}: {}", index, text);

    let api_key = std::env::var("OPENAI_API_KEY")
        .expect("Expected an API key for OpenAI in the environment variables");

    let response = client
        .post("https://api.openai.com/v1/audio/speech")
        .header("Authorization", format!("Bearer {}", api_key))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "model": tts_model,
            "voice": reading_voice,
            "input": text,
            "response_format": audio_format,
            "speed": speed,
        }))
        .send()
        .await;

    match response {
        Ok(resp) => {
            if let Ok(bytes) = resp.bytes().await {
                let _ = audio_tx.send((index, bytes)).await;
            }
        }
        Err(e) => println!("Failed to process audio for text: {}\nError: {:?}", text, e),
    }
}

async fn play_audio_from_queue(mut audio_rx: mpsc::Receiver<(usize, Bytes)>) {
    tokio::task::spawn_blocking(move || {
        let (_stream, stream_handle) = OutputStream::try_default().unwrap();
        let mut buffer = HashMap::new();
        let mut next_index = 0;

        while let Some((index, bytes)) = audio_rx.blocking_recv() {
            buffer.insert(index, bytes);
            while let Some(bytes) = buffer.remove(&next_index) {
                let cursor = Cursor::new(bytes);
                if let Ok(source) = Decoder::new_mp3(cursor) {
                    let sink = Sink::try_new(&stream_handle).unwrap();
                    sink.append(source.convert_samples::<f32>());
                    sink.sleep_until_end();
                }
                next_index += 1;
            }
        }
    })
    .await
    .unwrap();
}

async fn audio_to_output_file(mut audio_rx: mpsc::Receiver<(usize, Bytes)>, file_path: String) {
    let mut output_file = File::create(file_path).unwrap();
    while let Some((_index, bytes)) = audio_rx.recv().await {
        output_file.write_all(&bytes).unwrap();
    }
}

#[tokio::main]
async fn main() {
    env_logger::init();

    let cli = Cli::parse();

    let input_text = if cli.use_stdin {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .expect("Failed to read from stdin");
        buffer
    } else if cli.clipboard {
        let mut ctx: ClipboardContext = ClipboardProvider::new().unwrap();
        ctx.get_contents().unwrap_or_else(|_| {
            eprintln!("Failed to access clipboard contents.");
            std::process::exit(1);
        })
    } else if let Some(file) = cli.input_file {
        std::fs::read_to_string(file.clone()).unwrap_or_else(|_| {
            eprintln!("Failed to read file: {}", file);
            std::process::exit(1);
        })
    } else {
        Cli::command().error(
            clap::error::ErrorKind::MissingRequiredArgument,
            "No input source specified.",
        ).exit();
    };

    let output_file_format = match cli.format {
        Some(ResponseFormat::Opus) => "opus",
        Some(ResponseFormat::Aac) => "aac",
        Some(ResponseFormat::Flac) => "flac",
        Some(ResponseFormat::Pcm) => "pcm",
        Some(ResponseFormat::Mp3) => "mp3",
        _ => "mp3",
    };

    let reading_voice = match cli.voice {
        Some(Voice::Echo) => "echo",
        Some(Voice::Onyx) => "onyx",
        Some(Voice::Nova) => "nova",
        Some(Voice::Alloy) => "alloy",
        Some(Voice::Fable) => "fable",
        Some(Voice::Shimmer) => "shimmer",
        _ => "alloy",
    };

    let speed = cli.speed.unwrap_or(1.0); // Default speed
    let tts_model = if cli.hd { "tts-1-hd" } else { "tts-1" };

    let client = Client::new();
    let (audio_tx, audio_rx) = mpsc::channel::<(usize, Bytes)>(32);
    let chunks = split_input(&input_text, 4096);

    let handles: Vec<_> = chunks
        .iter()
        .enumerate()
        .map(|(index, text)| {
            let audio_tx = audio_tx.clone();
            let client = client.clone();
            let text = text.clone();
            let format = output_file_format;
            tokio::spawn(async move {
                fetch_and_process_audio(
                    &text,
                    index,
                    &client,
                    audio_tx,
                    &format,
                    reading_voice,
                    tts_model,
                    speed,
                )
                .await
            })
        })
        .collect();

    if let Some(output_file) = cli.output_file {
        tokio::spawn(audio_to_output_file(audio_rx, output_file));
    } else {
        tokio::spawn(play_audio_from_queue(audio_rx));
    }

    for handle in handles {
        let _ = handle.await;
    }

    // Drop the sender to close the channel and end the playback loop
    drop(audio_tx);
}
