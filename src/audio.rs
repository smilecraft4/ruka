use std::io::{stdout, Write};

use crate::error::{Error, Result};
use async_trait::async_trait;

use rustube::Video;

#[async_trait]
pub trait Downloader {
    async fn download(video: Video) -> Result<Vec<u8>>;
}

pub struct YoutubeDownloader;

#[async_trait]
impl Downloader for YoutubeDownloader {
    async fn download(video: Video) -> Result<Vec<u8>> {
        let best_audio = match video.best_audio() {
            Some(stream) => stream,
            None => return Err(Error::Generic(format!("Failed to get a audio file"))),
        };

        let mut tries = 0;

        let mut response = loop {
            tries += 1;

            match reqwest::get(best_audio.signature_cipher.url.clone()).await {
                Ok(response) => break Ok(response),
                Err(e) => {
                    if tries == 3 {
                        break Err(Error::Generic(format!(
                            "Failed to get response from {}, [error] {}",
                            best_audio.signature_cipher.url, e
                        )));
                    }
                }
            };
        }?;

        if response.status().is_success() {
            let content_length = response.content_length();

            let mut bytes = Vec::<u8>::new();
            if content_length.is_some() {
                bytes.reserve(content_length.unwrap() as usize);
            }

            let mut total_written = 0.0;
            while let Some(chunk) = response.chunk().await? {
                let chunk_size = chunk.len() as f64;
                bytes.extend(chunk);

                total_written += chunk_size;

                let progress = (total_written / content_length.unwrap() as f64) * 100.0;
                print!("\rProgress {:.2}% ({:.3})mb", progress, total_written / 1e6);
                stdout().flush()?;
            }
            println!("");

            return Ok(bytes);
        }

        Err(Error::Generic(format!("Failed to dowload file")))
    }
}

pub async fn dowload_cover_art(url: String) -> Result<(Vec<u8>, String)> {
    let response = reqwest::get(url).await?;

    // Check status
    if response.status().is_success() {
        if let Some(content_type) = response
            .headers()
            .get(reqwest::header::CONTENT_TYPE)
            .cloned()
        {
            //Check content type
            if content_type.to_str().unwrap().starts_with("image/") {
                // Get extension
                let extension = content_type
                    .to_str()
                    .unwrap()
                    .split('/')
                    .nth(1)
                    .unwrap_or("jpg");
                println!("Detected image extension: {}", extension);
                let bytes = response.bytes().await?;

                Ok((bytes.to_vec(), String::from(extension)))
            } else {
                Err(Error::Generic(format!(
                    "Cover art URL does not point to an image."
                )))
            }
        } else {
            Err(Error::Generic(format!(
                "Cover art content-Type header missing."
            )))
        }
    } else {
        Err(Error::Generic(format!(
            "Request was not successful: {}",
            response.status()
        )))
    }
}
