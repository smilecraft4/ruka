use crate::error::*;
use std::{collections::HashMap, io::Read, path::PathBuf, str::FromStr};

#[derive(Debug, Clone)]
pub struct Parameter {
    pub audio_url: String,
    pub cover_url: Option<String>,
    pub output: std::path::PathBuf,
    pub metadata: Option<HashMap<String, String>>,
    pub debug: bool,
}

#[derive(Debug, serde::Deserialize)]
pub struct Metadata {
    title: Option<String>,
    comment: Option<String>,
    description: Option<String>,
    artist: Option<Vec<String>>,
    album_artist: Option<Vec<String>>,
    album: Option<String>,
    date: Option<String>,
    track: Option<String>,
    track_total: Option<String>,
    disc: Option<String>,
    genre: Option<Vec<String>>,
    grouping: Option<String>,
    composer: Option<Vec<String>>,
    producer: Option<Vec<String>>,
    publisher: Option<Vec<String>>,
    copyright: Option<String>,
    author_url: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
pub struct ConfigJson {
    audio_url: String,
    cover_url: String,
    metadata: Metadata,
}

impl Parameter {
    pub fn from_args(args: &clap::ArgMatches) -> Result<Parameter> {
        let debug = args.get_flag("debug");

        // config mode
        let config_path = args.get_one::<String>("config");
        if config_path.is_some() {
            let config_path = config_path.unwrap().to_string();
            let config = crate::cli::ConfigJson::from_file(config_path)?;
            let mut param: Parameter = config.into();

            let output_dir = args.get_one::<String>("dir").unwrap();
            let output_format = args.get_one::<String>("format").unwrap();
            let output_name = match param.metadata.as_ref().unwrap().get("title") {
                Some(title) => title.to_string(),
                None => "dowload".to_string(),
            };

            param.output = PathBuf::from_str(output_dir)
                .unwrap()
                .join(output_name)
                .with_extension(output_format);
            param.debug = debug;

            return Ok(param);
        }

        todo!("implement simple mode")

        // simple mode
    }
}

pub fn parse_command_args() -> clap::ArgMatches {
    // simple mode
    let url_arg = clap::Arg::new("url")
        .long("url")
        .help("youtube url from where to dowload the audio")
        .value_name("URL");

    let output_arg = clap::Arg::new("output")
        .long("output")
        .help("Path to the downloaded audio")
        .value_hint(clap::ValueHint::FilePath)
        .value_name("FILE")
        .default_value("./download/track");

    // config mode
    let dir_arg = clap::Arg::new("dir")
        .long("dir")
        .help("Path to the downloaded audio")
        .value_hint(clap::ValueHint::DirPath)
        .value_name("DIR")
        .default_value("./download");

    let format_arg = clap::Arg::new("format")
        .long("format")
        .help("Format of the exported audio")
        .default_value("m4a");

    let config_arg = clap::Arg::new("config")
        .long("config")
        .value_hint(clap::ValueHint::FilePath)
        .value_name("FILE");

    // general tag
    let debug_arg = clap::Arg::new("debug")
        .short('d')
        .long("debug")
        .help("Turn on debug message")
        .action(clap::ArgAction::SetTrue);

    // command
    let args = clap::Command::new("ruka")
        .version("1.0.0") // TODO: use the cargo.toml
        .author("smilecraft4") // TODO: use the cargo.toml
        .about("Download song directly from youtbe to your pc, with medata and more")
        .args(vec![
            url_arg, output_arg, dir_arg, config_arg, format_arg, debug_arg,
        ])
        .get_matches();

    args
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
        let mut m = HashMap::<String, String>::new();

        let source = self.metadata;

        add_metadata(&mut m, "title".to_string(), &source.title);
        add_metadata(&mut m, "comment".to_string(), &source.comment);
        add_metadata(&mut m, "description".to_string(), &source.description);
        add_metadata(&mut m, "artist".to_string(), &source.artist);
        add_metadata(&mut m, "album_artist".to_string(), &source.album_artist);
        add_metadata(&mut m, "album".to_string(), &source.album);
        add_metadata(&mut m, "date".to_string(), &source.date);
        add_metadata(&mut m, "track".to_string(), &source.track);
        add_metadata(&mut m, "TRACKTOTAL".to_string(), &source.track_total);
        add_metadata(&mut m, "disc".to_string(), &source.disc);
        add_metadata(&mut m, "genre".to_string(), &source.genre);
        add_metadata(&mut m, "grouping".to_string(), &source.grouping);
        add_metadata(&mut m, "composer".to_string(), &source.composer);
        add_metadata(&mut m, "producer".to_string(), &source.producer);
        add_metadata(&mut m, "publisher".to_string(), &source.publisher);
        add_metadata(&mut m, "copyright".to_string(), &source.copyright);
        add_metadata(&mut m, "author_url".to_string(), &source.author_url);

        Parameter {
            audio_url: self.audio_url,
            cover_url: Some(self.cover_url),
            output: PathBuf::new(),
            debug: false,
            metadata: Some(m),
        }
    }
}

trait MetadataTag {
    fn to_string(&self) -> Option<String>;
}

impl MetadataTag for Option<String> {
    fn to_string(&self) -> Option<String> {
        self.clone()
    }
}

impl MetadataTag for Option<Vec<String>> {
    fn to_string(&self) -> Option<String> {
        if self.is_none() {
            return None;
        }

        let val = self.clone().unwrap();
        if val.len() <= 0 {
            return None;
        }

        let mut convert = format!("{}", val[0]);
        for val in val.iter().skip(1) {
            convert.push_str(format!(", {}", val).as_str());
        }

        Some(convert)
    }
}

fn add_metadata<T: MetadataTag>(metadata: &mut HashMap<String, String>, name: String, tag: &T) {
    let val = tag.to_string();
    if val.is_none() {
        return;
    }

    let val = val.unwrap();

    if val.len() > 0 {
        metadata.insert(name, val);
    }
}
