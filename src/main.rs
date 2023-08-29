#![allow(unused)]

use ruka::{audio::*, cli::*, converter::convert_to_mp3, error::*};
use std::{fs, io::Write, path::Path, str::FromStr};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Read CLI args

    // Get Info about the download mode (custom, direct, playlist)
    //

    // TODO: Support playlist mode
    // TODO: Support track mode
    // TODO: Support config mode

    // TODO: Fetch info about album
    // TODO: Fetch info about track

    let args = parse_command_args();
    let param = Parameter::from_args(&args)?;

    fs::create_dir_all(&param.output.parent().unwrap());

    let (cover, extension) = dowload_cover_art(param.cover_url.unwrap()).await?;

    let url = reqwest::Url::from_str(param.audio_url.as_str()).unwrap();
    let video = rustube::VideoFetcher::from_url(&url)?
        .fetch()
        .await?
        .descramble()?;
    let mut audio = YoutubeDownloader::download(video).await?;

    let mut temp_audio = tempfile::NamedTempFile::new()?;
    temp_audio.write_all(&audio)?;

    let mut temp_cover = tempfile::Builder::new()
        .suffix(format!(".{extension}").as_str())
        .tempfile()
        .expect("failed to create temporary cover art file");
    temp_cover.write_all(&cover)?;

    let audio_file = temp_audio.path().to_str().unwrap();
    let cover_file = temp_cover.path().to_str().unwrap();
    let output_file = param.output.to_str().unwrap();

    let mut process = std::process::Command::new("ffmpeg");

    #[rustfmt::skip]
          let command = process
          .args(&[
              "-y",
              "-i", audio_file,
              "-i",  cover_file,
              "-map", "0", "-map", "1",
              "-id3v2_version", "3",
          ]);

    // dbg!("{:#?}", &param.metadata);

    if param.metadata.is_some() {
        for (key, val) in param.metadata.unwrap() {
            let key = format!("{}={}", key, val);
            // println!("{key}");
            command.args(&["-metadata", key.as_str()]);
        }
    }

    // println!("{:?}", command);

    let info = command.arg(&output_file).output().expect("failed command");

    if info.status.success() {
        println!("Saved {output_file}");
    } else {
        println!("[Failed to set cover art]: {:#?}", info);
    }

    Ok(())
}
