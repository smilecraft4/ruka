#![allow(unused)]

use ffmpeg_next::ffi::AVFormatContext;
use ruka::{audio::*, cli::*, converter::convert_to_mp3, error::*};
use std::{fs, io::Write, path::Path};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = parse_command_args();
    let param = Parameter::from_args(&args)?;

    fs::create_dir_all(&param.output.parent().unwrap());

    // dowload cover art
    let cover = dowload_cover_art(param.cover_url.unwrap()).await?;
    let mut audio = YoutubeDowloader::dowload(param.audio_url).await?;

    convert_to_mp3(&mut audio, param.output, cover, param.metadata.unwrap())?;

    Ok(())
}
