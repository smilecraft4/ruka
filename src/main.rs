#![allow(unused)]

use std::io::Write;

use ruka::{
    audio::{
        Dowloader, {self, dowload_cover_art, YoutubeDowloader},
    },
    cli::{parse_command_args, Parameter},
    error::Result,
    prelude::Error,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = parse_command_args();
    let param = Parameter::from_args(&args)?;

    // dowload cover art
    let cover = dowload_cover_art(param.cover_url.unwrap()).await?;
    let audio = YoutubeDowloader::dowload(param.audio_url).await?;
    // TODO: metadata to dictionary

    Ok(())
}
