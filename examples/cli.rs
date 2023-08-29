use std::{
    future::Future,
    io::{stdout, Write},
};

use rand::Rng;

const RUKA_CLI_NAME: &str = "ruka";
const RUKA_CLI_VERSION: &str = "0.0.1";
const RUKA_CLI_AUTHOR: &str = "smilecraft4";
const RUKA_CLI_ABOUT: &str = "Ruka is a cli tool to download songs and album from url";
const RUKA_CLI_LONGABOUT: &str = r#"
Ruka is a cli tool to download songs and album from url.
It allows to download video and playlist from youtube and 
convert them to the desired audio format such as .m4a, .mp3 or .wav
"#;

const LOADING_CHAR: [char; 4] = ['-', '\\', '|', '/'];

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let matches = clap::Command::new(RUKA_CLI_NAME)
        .about(RUKA_CLI_ABOUT)
        .version(RUKA_CLI_VERSION)
        .author(RUKA_CLI_AUTHOR)
        .long_about(RUKA_CLI_LONGABOUT)
        .args([
            clap::Arg::new("url").required(true).long("url"),
            clap::Arg::new("output").required(false).long("output"),
            clap::Arg::new("album").required(false).long("album"),
            clap::Arg::new("artist").required(false).long("artist"),
        ])
        .get_matches();

    let url = matches.get_one::<String>("url");
    let output = matches.get_one::<String>("output");

    let artist = matches.get_one::<String>("artist");
    let album = matches.get_one::<String>("album");

    let search = match (album, artist) {
        (Some(a), Some(b)) => Some(format!("{a} by {b}")),
        (Some(a), None) => Some(format!("{a}")),
        (None, None) => None,
        _ => None,
    };

    let mut progress = 0.0;
    if search.is_some() {
        let search = search.unwrap();
        let start = std::time::Instant::now();
        let a = download_a(&search);

        loop {}

        let elapsed = std::time::Instant::elapsed(&start).as_micros();
        let index = elapsed as usize % LOADING_CHAR.len();
        print!(
            "\rRuka is searching for \"{}\"{} {:.2}%",
            &search, LOADING_CHAR[index], progress
        );
        stdout().flush();
    }

    Ok(())
}

async fn download_a(search: &String) -> Result<(), String> {
    let mut progress = 0.0;
    while progress < 100.0 {
        random_sleep(0.1, 0.5);
        progress += 1.0;
    }

    Ok(())
}

fn random_sleep(min: f64, max: f64) {
    let mut rand = rand::thread_rng();
    let time = rand.gen_range(min..max);

    std::thread::sleep(std::time::Duration::from_secs_f64(time));
}
