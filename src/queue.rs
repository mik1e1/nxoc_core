use rand::seq::SliceRandom;

use crate::{playlist::Playlist, track::Track};

#[derive(Default, Debug)]
pub struct Queue {
    tracks: Vec<Track>,
    cursor: Option<usize>,
}

impl From<Playlist> for Queue {
    fn from(playlist: Playlist) -> Self {
        Queue {
            tracks: playlist.tracks.into(),
            cursor: None,
        }
    }
}

impl Queue {
    pub fn new(tracks: Vec<Track>) -> Self {
        Queue {
            tracks,
            cursor: None,
        }
    }

    pub fn iter_cloned(&self) -> impl Iterator<Item = Track> {
        self.tracks.clone().into_iter()
    }

    pub fn is_empty(&self) -> bool {
        self.tracks.is_empty()
    }

    pub fn len(&self) -> usize {
        self.tracks.len()
    }

    pub fn track(&self, index: usize) -> Option<&Track> {
        if index < self.tracks.len() {
            return Some(&self.tracks[index]);
        }

        None
    }

    pub fn current_track(&self) -> Option<&Track> {
        if let Some(index) = self.cursor {
            return Some(&self.tracks[index]);
        }

        None
    }

    pub fn select(&mut self, index: usize) {
        if index <= self.tracks.len() {
            self.cursor = Some(index);
        }
    }

    pub fn cursor(&self) -> Option<usize> {
        self.cursor
    }

    pub fn enqueue(&mut self, track: Track) {
        self.tracks.push(track);
    }

    pub fn advance(&mut self) -> Option<&Track> {
        if !self.tracks.is_empty() {
            if let Some(index) = self.cursor {
                if index < self.tracks.len() - 1 {
                    self.cursor = Some(index.saturating_add(1));
                    return self.current_track();
                }
            } else {
                self.cursor = Some(0);
                return self.current_track();
            }
        }

        None
    }

    pub fn shuffle(&mut self) {
        if !self.tracks.is_empty() {
            let current_track = if let Some(track) = self.current_track() {
                track.clone()
            } else {
                self.tracks[0].clone()
            };

            let mut rng = rand::rng();
            self.tracks.shuffle(&mut rng);

            self.cursor = self.tracks.iter().position(|el| *el == current_track);
        }
    }
}
