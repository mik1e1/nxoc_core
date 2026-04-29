use serde::{Deserialize, Serialize};
use std::{path::PathBuf, time::Duration};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrackMetadata {
    pub duration: Duration,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Track {
    pub path: PathBuf,
    pub metadata: Option<TrackMetadata>,
}

impl From<PathBuf> for Track {
    fn from(path: PathBuf) -> Self {
        Track {
            path,
            metadata: None,
        }
    }
}
