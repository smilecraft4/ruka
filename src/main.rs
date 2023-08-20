#![allow(unused)]

use std::io::Write;

use ruka::{
    cli::{parse_command_args, Parameter},
    error::Result,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = parse_command_args();
    let param = Parameter::from_args(&args)?;

    dbg!(&param);

    // dowload cover art
    if param.cover_url.is_some() {
        let response = reqwest::get(param.cover_url.unwrap()).await?;

        // Check status
        if response.status().is_success() {
            if let Some(content_type) = response
                .headers()
                .get(reqwest::header::CONTENT_TYPE)
                .cloned()
            {
                //Check content type
                if content_type.to_str()?.starts_with("image/") {
                    // Get extension
                    let extension = content_type.to_str()?.split('/').nth(1).unwrap_or("jpg");
                    println!("Detected image extension: {}", extension);
                    let bytes = response.bytes().await?;

                    let mut file =
                        std::fs::File::create((format!("./target/cover.{}", extension)))?;
                    file.write_all(bytes.as_ref())?;
                } else {
                    println!("The URL does not point to an image.");
                }
            } else {
                println!("Content-Type header missing.");
            }
        } else {
            println!("Request was not successful: {}", response.status());
        }
    }

    // read link
    // dowload audio link
    // act based on the domain "youtube.com" -> youtbe audio dowloader

    // decode audio byte stream

    // add covert art to ffmpeg stream
    // add metadata to ffmpeg stream

    // encode to mp3

    Ok(())
}
