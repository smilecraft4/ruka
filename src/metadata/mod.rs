use crate::error::Result;

#[derive(Debug)]
pub struct MetadataTag<T> {
    id: String,
    value: Option<T>,
}

#[derive(Debug)]
pub struct Metadata {
    title: MetadataTag<String>,
    artist: MetadataTag<Vec<String>>,
    album: MetadataTag<String>,
    album_artist: MetadataTag<Vec<String>>,
    year: MetadataTag<String>,
    genre: MetadataTag<Vec<String>>,
    track_number: MetadataTag<String>,
    composer: MetadataTag<Vec<String>>,
    isrc: MetadataTag<String>,
    artwork: MetadataTag<String>,
}

impl<T> MetadataTag<T> {
    pub fn new(id: impl Into<String>, value: Option<T>) -> Self {
        MetadataTag {
            id: id.into(),
            value,
        }
    }
}

impl Metadata {
    pub fn new() -> Self {
        Self {
            title: MetadataTag::new("TIT2", None::<String>),
            artist: MetadataTag::new("TPE1", None::<Vec<String>>),
            album: MetadataTag::new("TALB", None::<String>),
            album_artist: MetadataTag::new("TPE2", None::<Vec<String>>),
            year: MetadataTag::new("TYER", None::<String>),
            genre: MetadataTag::new("TCON", None::<Vec<String>>),
            track_number: MetadataTag::new("TRCK", None::<String>),
            composer: MetadataTag::new("TCOM", None::<Vec<String>>),
            isrc: MetadataTag::new("TSRC", None::<String>),
            artwork: MetadataTag::new("APIC", None::<String>),
        }
    }

    pub fn from_json(json: String) -> Result<Self> {
        let mut data = Metadata::new();

        data.album.value = Some(json.clone());
        data.title.value = Some(String::from("title example"));

        Ok(data)
    }
}
