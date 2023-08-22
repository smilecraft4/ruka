use std::{io::Write, path::PathBuf, str::FromStr};

use clap::Arg;
use regex::Regex;
use reqwest;
use ruka::audio::Dowloader;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let command = clap::Command::new("ruka")
        .version("1.0.0") // TODO: use the cargo.toml
        .author("smilecraft4") // TODO: use the cargo.toml
        .about("Download song directly from youtbe to your pc, with medata and more")
        .args(vec![
            Arg::new("playlist")
                .short('p')
                .long("playlist")
                .help("Link to the youtube playlist")
                .required(true),
            Arg::new("format")
                .short('f')
                .long("format")
                .default_value("mp3")
                .help("output format of the audio"),
        ])
        .get_matches();

    let playlist_url = command.get_one::<String>("playlist").unwrap().clone();
    let audio_format = command.get_one::<String>("format").unwrap().clone();

    let response = reqwest::get(playlist_url).await?;
    let body = response.text().await?;

    let rex = Regex::new(r#""playlistVideoRenderer":\{"videoId":"(.*?)""#)?;

    let output_dir = PathBuf::from_str("./download/playlist")?;
    std::fs::create_dir_all(&output_dir)?;

    for capture in rex.captures_iter(&body) {
        let id = &capture[1].to_string();

        let url = format!("https://www.youtube.com/watch?v={}", id);

        let url = reqwest::Url::from_str(url.as_str()).unwrap();
        let video = rustube::VideoFetcher::from_url(&url)?
            .fetch()
            .await?
            .descramble()?;

        let output = PathBuf::from_str(
            format!(
                "{}/{}.{}",
                output_dir.as_path().to_str().unwrap(),
                video.title(),
                audio_format
            )
            .as_str(),
        )?;

        println!("working on: {:?}", output);

        let audio = ruka::audio::YoutubeDowloader::dowload(video).await?;
        let mut temp_audio = tempfile::NamedTempFile::new()?;
        temp_audio.write_all(&audio)?;

        let audio_file = temp_audio.path().to_str().unwrap();
        let output_file = output.as_path().to_str().unwrap();

        let mut process = std::process::Command::new("ffmpeg");

        let command = process
            .args(&["-y", "-i", audio_file, output_file])
            .output()
            .expect("failed command");

        if command.status.success() {
            println!("Saved {output_file}");
        } else {
            println!("[Failed to set cover art]: {:#?}", command);
        }
    }

    Ok(())
}
