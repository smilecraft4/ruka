use clap::Parser;
use futures_util::StreamExt;
use rsmpeg::{avformat::*, ffi::avdevice_register_all};
use rustube::*;
use std::io::prelude::*;

#[derive(Parser, Debug)]
#[command(name = "smilecraft4", version = "0.0.1", about = "youtube to mp3")]
struct Args {
    #[arg(short, long)]
    url: String,
    #[arg(short, long)]
    output: Option<String>,
    #[arg(short, long)]
    metadata: Option<String>,
    #[arg(short, long)]
    cover: Option<String>,
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    println!("Hello {}", args.url);
    println!("Hello {}", args.metadata.unwrap_or(String::from("None")));
    println!("Hello {}", args.cover.unwrap_or(String::from("None")));

    let id = Id::from_raw(&args.url).unwrap();
    let video = Video::from_id(id.into_owned()).await.unwrap();

    let mut file = std::fs::File::create("output.json").unwrap();

    writeln!(file, "{{\"{}\": [", video.title()).unwrap();
    for stream in video.streams().iter() {
        let str = serde_json::to_string_pretty(stream).unwrap();
        writeln!(file, "{},", str).unwrap();
    }
    writeln!(file, "]").unwrap();
    let audio_url = video
        .streams()
        .iter()
        .filter(|stream| {
            if !stream.codecs.contains(&String::from("opus")) {
                return false;
            }
            Some(stream);

            // let audio_quality = stream.audio_quality.unwrap();
            // if audio_quality == AudioQuality::High {
            //     println!("AudioQuality::High");
            //     Some(stream);
            // } else if audio_quality == AudioQuality::Medium {
            //     println!("AudioQuality::Medium");
            //     Some(stream);
            // } else if audio_quality == AudioQuality::Low {
            //     println!("AudioQuality::Low");
            //     Some(stream);
            // }

            return true;
        })
        .max_by(|a, b| {
            let audio_qulaity_a = a.audio_quality.unwrap();
            let audio_qulaity_b = b.audio_quality.unwrap();

            audio_qulaity_a.cmp(&audio_qulaity_b)
        })
        .unwrap();

    let url = audio_url.signature_cipher.url.to_owned();
    println!("{}", url);

    // Download cool file
    let client = reqwest::Client::new();
    let response = client.get(url).send().await.unwrap();

    // if !response.status().is_success() {
    //     panic!("dqsdqd");
    // }

    let content_length = response.content_length().unwrap_or(0);
    let mut total_written = 0;
    let mut dest = std::fs::File::create(video.title().to_ascii_lowercase()).unwrap();

    let mut str = response.bytes_stream();
    while let Some(item) = str.next().await {
        let chunk = item.unwrap();
        dest.write_all(&chunk).unwrap();
        total_written += chunk.len() as u64;

        if content_length > 0 {
            let progress = (total_written as f64 / content_length as f64) * 100.0;
            println!("Progress: {:.2}% ({} bytes)", progress, total_written);
        }
    }

    println!("Data has been written");

    // let path = rustube::download_best_quality(&args.url).await?;
    // println!("{}", path.display());
}
