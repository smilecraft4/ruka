use std::{collections::HashMap, io::Read};

use crate::error::{Error, Result};

use super::Parameter;

#[derive(Debug, serde::Deserialize)]
pub struct Metadata {
    title: Option<String>,
    artists: Option<Vec<String>>,
    album: Option<String>,
    album_artists: Option<Vec<String>>,
    year: Option<String>,
    genres: Option<Vec<String>>,
    track_number: Option<String>,
    composer: Option<Vec<String>>,
}

#[derive(Debug, serde::Deserialize)]
pub struct ConfigJson {
    audio_url: String,
    cover_url: String,
    metadata: Metadata,
}

impl ConfigJson {
    pub fn from_file(path: String) -> Result<ConfigJson> {
        let mut file = std::fs::File::open(path)?;

        let mut json_content = String::new();
        file.read_to_string(&mut json_content)?;

        let parsed: ConfigJson = match serde_json::from_str(&json_content) {
            Ok(s) => s,
            Err(_) => return Err(Error::Generic(format!("JSON parsing failed"))),
        };

        Ok(parsed)
    }
}

impl Into<Parameter> for ConfigJson {
    fn into(self) -> Parameter {
        let mut metadata = HashMap::<String, String>::new();

        let source = self.metadata;

        if source.title.is_some() {
            metadata.insert("title".to_string(), source.title.unwrap());
        }
        if source.artists.is_some() {
            match convert_vector_to_string(source.artists.unwrap()) {
                Some(val) => metadata.insert("artist".to_string(), val),
                None => None,
            };
        }
        if source.album.is_some() {
            metadata.insert("album".to_string(), source.album.unwrap());
        }
        if source.album_artists.is_some() {
            match convert_vector_to_string(source.album_artists.unwrap()) {
                Some(val) => metadata.insert("album_artist".to_string(), val),
                None => None,
            };
        }
        if source.year.is_some() {
            metadata.insert("year".to_string(), source.year.unwrap());
        }
        if source.genres.is_some() {
            match convert_vector_to_string(source.genres.unwrap()) {
                Some(val) => metadata.insert("genres".to_string(), val),
                None => None,
            };
        }
        if source.track_number.is_some() {
            metadata.insert("track_number".to_string(), source.track_number.unwrap());
        }
        if source.composer.is_some() {
            match convert_vector_to_string(source.composer.unwrap()) {
                Some(val) => metadata.insert("composer".to_string(), val),
                None => None,
            };
        }

        Parameter {
            audio_url: self.audio_url,
            cover_url: Some(self.cover_url),
            output: None,
            debug: false,
            metadata: Some(metadata),
        }
    }
}

fn convert_vector_to_string(values: Vec<String>) -> Option<String> {
    let mut convert = String::new();

    if values.len() <= 0 {
        return None;
    }

    convert.push_str(format!("{}", values[0]).as_str());
    for val in values.iter().skip(1) {
        convert.push_str(format!(", {}", val).as_str());
    }

    Some(convert)
}
