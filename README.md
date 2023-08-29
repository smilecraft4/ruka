# Ruka

<!-- TODO -->

<p align="center">
    <img src="https://www.nautiljon.com/images/perso/00/85/sarashina_ruka_19558.webp" alt="Ruka"/>
</p>

a simple CLI to download youtube music videos

use a json file to configures tags and metadata
dowload the thumbnails of the video
take a screenshot of the video and set as thumbnail

## Functionality

Support multiple mode such as Playlist

```console
$ ruka --url "https://www.url.com/id" --output "~/Documents/" --format mp3
```

You can add more info and data to be automatically written to the file. This will automatically get all the relevant information's about the song metadata such as year, artist, cover art, ... and ask the user to choose the correct information's


```console
$ ruka --url "https://www.url.com/id" --album "Love War" --album_artist "Yena"
Ruka is searching for "Love War by Yena"
Ruka found 3 album for: "Love War by Yena"
1. Love War, Yena (2023) - 3 track:
    - Intro
    - Wash Away
    x Love War (missing)

Choose the album: [default: 1]
$ 1
Ruka is downloading Cover Art...100%
Ruka is downloading Audio... 91% 


```