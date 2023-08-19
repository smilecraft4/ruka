use crate::error::Result;

use std::str::FromStr;

#[derive(Debug)]
pub struct Parameter {
    pub url: String,
    pub output: Option<std::path::PathBuf>,
    pub debug: bool,
}

impl Parameter {
    pub fn from_args(args: &clap::ArgMatches) -> Result<Parameter> {
        let url = args.get_one::<String>("url").unwrap().to_owned();

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

        Ok(Parameter { url, output, debug })
    }
}

pub fn parse_command_args() -> clap::ArgMatches {
    let arg_url = clap::Arg::new("url")
        .short('u')
        .long("url")
        .help("youtube url from where to dowload the audio")
        .value_name("URL")
        .required(true);

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

    let args = clap::Command::new("Ruka-dl")
        .version("1.0.0") // TODO: use the cargo.toml
        .author("smilecraft4") // TODO: use the cargo.toml
        .about("Download song directly from youtbe to your pc, with medata and more")
        .args(vec![arg_url, output_arg, debug_arg])
        .get_matches();

    args
}
