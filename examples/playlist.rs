use std::{
    collections::HashSet,
    io::{BufRead, Write},
    path::PathBuf,
    str::FromStr,
};

use clap::{Arg, ValueHint};
use regex::Regex;
use reqwest;
use ruka::{
    audio::{Downloader, YoutubeDownloader},
    error::Result,
    prelude::Error,
};

#[tokio::main]
async fn main() -> Result<()> {
    // TODO: Add a HashSet to track the progress of the playlist to avoid restarting from zero

    let command = clap::Command::new("ruka")
        .version("1.0.0") // TODO: use the cargo.toml
        .author("smilecraft4") // TODO: use the cargo.toml
        .about("Download song directly from youtube to your pc, with metadata and more")
        .args(vec![
            Arg::new("playlist")
                .short('p')
                .long("playlist")
                .help("Link to the youtube playlist")
                .value_name("URL")
                .required(true),
            Arg::new("format")
                .short('f')
                .long("format")
                .default_value("mp3")
                .help("output format of the audio"),
            Arg::new("output")
                .short('o')
                .long("output")
                .default_value("./download/playlist")
                .value_hint(ValueHint::DirPath)
                .value_name("PATH")
                .help("Directory where tracks will be saved"),
        ])
        .get_matches();

    let playlist_url = command.get_one::<String>("playlist").unwrap().clone();
    let audio_format = command.get_one::<String>("format").unwrap().clone();
    let output_string = command.get_one::<String>("output").unwrap().clone();

    let response = reqwest::get(playlist_url).await?;
    let body = response.text().await?;

    let rex = Regex::new(r#""playlistVideoRenderer":\{"videoId":"(.*?)""#)?;

    std::fs::create_dir_all(&output_string.as_str())?;

    let mut playlist_urls = Vec::<String>::new();
    for capture in rex.captures_iter(&body) {
        let url = format!("https://www.youtube.com/watch?v={}", capture[1].to_string());
        playlist_urls.push(url);
    }

    // load file of progress and parse it to a HashSet

    let progress_file_path = format!("{}/{}", output_string, "progress.txt");
    let progress = match load_progress_file(&progress_file_path) {
        Ok(progress) => {
            println!(
                "Loaded previous progress file {} with {} links",
                &progress_file_path,
                progress_file_path.len()
            );
            progress
        }
        Err(_) => {
            println!("Starting from beginning because a progress file was not found");
            HashSet::<String>::new()
        }
    };

    download_urls(progress, playlist_urls, (output_string, audio_format)).await?;
    std::fs::remove_file(progress_file_path)?;

    Ok(())
}

async fn download_urls(
    mut progress: HashSet<String>,
    urls: Vec<String>,
    (dir, format): (String, String),
) -> ruka::error::Result<()> {
    let failed_url_path = format!("{}/{}", dir, "failed.txt");
    let mut failed_url = Vec::<String>::new();

    let field = progress.clone();
    let urls: Vec<String> = urls.into_iter().filter(|u| !field.contains(u)).collect();

    for url_string in urls {
        println!("Downloading {}", &url_string);
        let url = match reqwest::Url::from_str(&url_string.clone()) {
            Ok(url) => url,
            Err(e) => {
                println!("failed to parse url {}: {}", &url_string, e);
                failed_url.push(url_string);

                continue;
            }
        };

        let video = match rustube::VideoFetcher::from_url(&url)?.fetch().await {
            Ok(v) => v,
            Err(e) => {
                println!("failed to get youtube video {}: {}", &url_string, e);
                failed_url.push(url_string);

                continue;
            }
        }
        .descramble()?;

        let output = format!("{}/{}.{}", dir, video.title().to_ascii_lowercase(), format);
        let output = match PathBuf::from_str(&output) {
            Ok(out) => out,
            Err(e) => return Err(Error::Generic(format!("Failed to get directory: {}", e))),
        };

        let audio = YoutubeDownloader::download(video).await?;

        // create temporary file to store stream of data
        let mut temp_audio = tempfile::NamedTempFile::new()?;
        temp_audio.write_all(&audio)?;

        // path
        let audio_file = temp_audio.path().to_str().unwrap();
        let output_file = output.as_path().to_str().unwrap();

        let mut process = std::process::Command::new("ffmpeg");

        let command = process
            .args(&["-y", "-i", audio_file, output_file])
            .output()
            .expect("failed command");

        progress.insert(url.to_string());
        let progress_file_path = format!("{}/{}", dir, "progress.txt");
        save_progress_file(&progress_file_path, progress.clone())?;

        if command.status.success() {
            println!("Saved {output_file}");
        } else {
            println!("[Failed to set cover art]: {:#?}", command);
        }
    }

    save_failed_file(&failed_url_path, &failed_url)?;

    Ok(())
}

fn save_progress_file(path: &String, progress: HashSet<String>) -> Result<()> {
    let mut file = std::fs::File::create(path)?;

    for url in progress.iter() {
        file.write_fmt(format_args!("{}\n", url.to_string()))?;
    }

    Ok(())
}

fn save_failed_file(path: &String, failed: &Vec<String>) -> Result<()> {
    let mut file = std::fs::File::create(path)?;

    for url in failed.iter() {
        file.write_fmt(format_args!("{}\n", url))?;
    }

    Ok(())
}

fn load_progress_file(path: &String) -> Result<HashSet<String>> {
    let file = std::fs::File::open(path)?;
    let buf = std::io::BufReader::new(file);

    let mut progress_hash = HashSet::<String>::new();
    for line_result in buf.lines() {
        let line = line_result?;
        progress_hash.insert(line);
    }

    Ok(progress_hash)
}
