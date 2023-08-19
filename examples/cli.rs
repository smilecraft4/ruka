use std::str::FromStr;

use clap::ArgMatches;

#[derive(Debug)]
pub struct Parameter {
    /// TODO change this to a strategy pattern where user can implement their own dowloading for services
    pub url: reqwest::Url,
    pub output: Option<std::path::PathBuf>,
    pub debug: bool,
}

impl Parameter {
    pub fn from_args(args: &clap::ArgMatches) -> Result<Parameter, String> {
        // Bad Url
        // Url not supported

        // parse url to correct type
        let url_string = args.get_one::<String>("url").unwrap();
        let url = reqwest::Url::parse(url_string).unwrap();
        if url.domain() != Some("www.youtube.com") {
            return Err(format!(
                "songs from {} are not supported",
                url.domain().unwrap(),
            ));
        }

        let output = match args.get_one::<String>("output") {
            Some(p) => Some(std::path::PathBuf::from_str(p).unwrap()),
            None => None,
        };

        let debug = args.get_flag("debug");

        Ok(Parameter { url, output, debug })
    }
}

fn command_args() -> ArgMatches {
    let arg_url = clap::Arg::new("url")
        .short('u')
        .long("url")
        .help("song youtube url to download")
        .value_name("URL")
        .required(true);

    let output_arg = clap::Arg::new("output")
        .short('o')
        .long("output")
        .help("Path to the downloaded song")
        .value_hint(clap::ValueHint::FilePath)
        .value_name("FILE")
        .required(false);

    let debug_arg = clap::Arg::new("debug")
        .short('d')
        .long("debug")
        .help("Turn on debug message")
        .action(clap::ArgAction::SetTrue)
        .required(false);

    let args = clap::Command::new("Ruka-dl")
        .version("0.2.0")
        .author("smilecraft4")
        .about("Download song directly from youtbe to your pc, with medata and more")
        .args(vec![arg_url, output_arg, debug_arg])
        .get_matches();

    args
}

fn main() {
    let args = command_args();
    let param = Parameter::from_args(&args);

    dbg!(param).unwrap();
}
