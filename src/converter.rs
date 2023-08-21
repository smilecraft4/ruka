use std::{collections::HashMap, path::PathBuf};

use crate::error::Result;

pub fn convert_to_mp3(
    _audio_source: &mut Vec<u8>,
    _path: PathBuf,
    _image: Vec<u8>,
    _metadata: HashMap<String, String>,
) -> Result<()> {
    Ok(())
}
