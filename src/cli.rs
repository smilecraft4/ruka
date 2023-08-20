use crate::error::{Error, Result};
use std::{collections::HashMap, io::Read, str::FromStr};

#[derive(Debug)]
pub struct Parameter {
    pub audio_url: String,
    pub cover_url: Option<String>,
    pub output: Option<std::path::PathBuf>,
    pub debug: bool,
    pub metadata: Option<HashMap<String, String>>,
}

impl Parameter {
    pub fn from_args(args: &clap::ArgMatches) -> Result<Parameter> {
        let output = match args.get_one::<String>("output") {
            Some(p) => {
                let mut output = std::path::PathBuf::from_str(p).unwrap();
                if output.extension().is_none() {
                    // TODO: provide in configs a default format
                    output.set_extension("mp3");
                }

                Some(output)
            }
            None => None,
        };

        let debug = args.get_flag("debug");

        // TODO: if "--config" file is on disable --url

        let config_path = args.get_one::<String>("config");
        if config_path.is_some() {
            let config_path = config_path.unwrap().to_owned();

            let config = crate::cli::ConfigJson::from_file(config_path)?;
            let mut param: Parameter = config.into();
            param.output = output;
            param.debug = debug;

            println!("Config mode");

            return Ok(param);
        } else {
            let url = args.get_one::<String>("url").unwrap().to_owned();

            println!("Simple mode");

            Ok(Parameter {
                audio_url: url,
                cover_url: None,
                output,
                debug,
                metadata: None,
            })
        }
    }
}

pub fn parse_command_args() -> clap::ArgMatches {
    let url_arg = clap::Arg::new("url")
        .short('u')
        .long("url")
        .help("youtube url from where to dowload the audio")
        .value_name("URL");

    let output_arg = clap::Arg::new("output")
        .short('o')
        .long("output")
        .help("Path to the downloaded audio")
        .value_hint(clap::ValueHint::FilePath)
        .value_name("FILE")
        .required(false);

    let debug_arg = clap::Arg::new("debug")
        .short('d')
        .long("debug")
        .help("Turn on debug message")
        .action(clap::ArgAction::SetTrue)
        .required(false);

    let config_arg = clap::Arg::new("config")
        .short('c')
        .long("config")
        .value_hint(clap::ValueHint::FilePath);

    let args = clap::Command::new("ruka")
        .version("1.0.0") // TODO: use the cargo.toml
        .author("smilecraft4") // TODO: use the cargo.toml
        .about("Download song directly from youtbe to your pc, with medata and more")
        .args(vec![url_arg, output_arg, debug_arg, config_arg])
        .get_matches();

    args
}

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
