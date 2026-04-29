use serde::{Deserialize, Serialize};

use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::PathBuf,
};

use crate::{
    filesystem::{LIBRARY_RELATIVE_PATH, Storable},
    track::Track,
};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Playlist {
    pub name: String,
    pub tracks: Vec<Track>,
}

impl Storable for Playlist {
    fn save(&self) -> anyhow::Result<()> {
        let path: PathBuf = [
            dirs::config_dir().unwrap(),
            PathBuf::from(LIBRARY_RELATIVE_PATH),
        ]
        .iter()
        .collect();
        let content = toml::to_string(&self)?;

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }

        let mut file = OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(false)
            .open(&path)?;

        file.write_all(content.as_bytes())?;

        Ok(())
    }

    fn load() -> anyhow::Result<Playlist> {
        let path: PathBuf = [
            dirs::config_dir().unwrap(),
            PathBuf::from(LIBRARY_RELATIVE_PATH),
        ]
        .iter()
        .collect();
        let content = fs::read(&path)?;

        Ok(toml::from_slice(&content)?)
    }
}
