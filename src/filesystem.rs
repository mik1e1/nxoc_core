use std::{
    fs,
    os::unix::ffi::OsStrExt,
    path::{Path, PathBuf},
};

use anyhow::anyhow;

use crate::audio::SUPPORTED_FORMATS;

pub trait Storable {
    fn save(&self) -> anyhow::Result<()>;
    fn load() -> anyhow::Result<Self>
    where
        Self: Sized;
}

// multiple playlists, id est library
pub const LIBRARY_RELATIVE_PATH: &str = "nxoc/data/library.toml";

#[derive(Debug, Clone)]
pub struct Folder {
    pub path: PathBuf,
    pub children: Vec<AudioNode>,
}

#[derive(Debug, Clone)]
pub struct AudioFile {
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub enum AudioNode {
    Folder(Folder),
    AudioFile(AudioFile),
}

impl TryFrom<&Path> for AudioNode {
    type Error = anyhow::Error;

    fn try_from(path: &Path) -> anyhow::Result<AudioNode> {
        if path.is_file() {
            let ext_opt = path.extension().and_then(|x| x.to_str());

            if let Some(ext) = ext_opt
                && SUPPORTED_FORMATS.iter().any(|fmt| *fmt == ext)
            {
                return Ok(AudioNode::AudioFile(AudioFile {
                    path: path.to_path_buf(),
                }));
            }
        }

        if path.is_dir() {
            let children: Vec<AudioNode> = fs::read_dir(path)?
                .filter_map(|entry| entry.ok())
                .filter_map(|entry| AudioNode::try_from(entry.path().as_path()).ok())
                .collect();

            return Ok(AudioNode::Folder(Folder {
                path: path.to_path_buf(),
                children,
            }));
        }

        Err(anyhow!(
            "Unsupported file format or nonexistent path: {:?}",
            path
        ))
    }
}

impl AudioNode {
    pub fn is_track(&self) -> bool {
        match self {
            AudioNode::AudioFile(_) => true,
            AudioNode::Folder(_) => false,
        }
    }

    pub fn folders(&self) -> usize {
        self.iter()
            .filter_map(AudioNode::as_folder)
            .fold(0, |acc, _| acc + 1)
    }

    pub fn as_folder(&self) -> Option<&Folder> {
        match self {
            AudioNode::Folder(f) => Some(f),
            _ => None,
        }
    }

    pub fn path(&self) -> &Path {
        match self {
            AudioNode::AudioFile(af) => &af.path,
            AudioNode::Folder(f) => &f.path,
        }
    }

    pub fn iter(&self) -> Box<dyn Iterator<Item = &AudioNode> + '_> {
        match self {
            AudioNode::AudioFile(_) => Box::new(std::iter::once(self)),
            AudioNode::Folder(f) => {
                Box::new(std::iter::once(self).chain(f.children.iter().flat_map(|x| x.iter())))
            }
        }
    }

    pub fn prune(self) -> Option<AudioNode> {
        match self {
            AudioNode::AudioFile(_) => Some(self),
            AudioNode::Folder(mut f) => {
                f.children = f
                    .children
                    .into_iter()
                    .filter_map(|entry| entry.prune())
                    .collect();

                if f.children.is_empty() {
                    None
                } else {
                    Some(AudioNode::Folder(f))
                }
            }
        }
    }
}

pub fn scan_dir_rec(path: &Path) -> anyhow::Result<impl Iterator<Item = PathBuf>> {
    fn inner(path: &PathBuf) -> anyhow::Result<Box<dyn Iterator<Item = PathBuf>>> {
        match fs::read_dir(path) {
            Ok(rd) => {
                let mut entries: Vec<_> = rd.filter_map(|e| e.ok()).collect();
                entries.sort_by_key(|e| e.file_name());

                let rec_obj = entries.into_iter().flat_map(|e| {
                    let path = e.path();
                    std::iter::once(path.clone()).chain({
                        if path.is_dir()
                            && let Ok(inner_iter) = inner(&path)
                        {
                            inner_iter
                        } else {
                            Box::new(std::iter::empty())
                        }
                    })
                });
                Ok(Box::new(rec_obj))
            }
            Err(e) => Err(e.into()),
        }
    }

    inner(&path.to_path_buf())
}

pub fn scan_dir(path: &Path) -> anyhow::Result<impl Iterator<Item = PathBuf>> {
    match fs::read_dir(path) {
        Ok(rd) => Ok(rd.filter_map(|rd| rd.ok()).map(|e| e.path())),
        Err(e) => Err(e.into()),
    }
}

pub fn list_dirs_rec(path: &Path) -> anyhow::Result<impl Iterator<Item = PathBuf>> {
    fn inner(path: &Path) -> anyhow::Result<Box<dyn Iterator<Item = PathBuf>>> {
        match fs::read_dir(path) {
            Ok(rd) => Ok(Box::new(rd.filter_map(|e| e.ok()).flat_map(|e| {
                let path = e.path();
                std::iter::once(path.clone()).chain({
                    if path.is_dir()
                        && e.file_name().as_os_str().as_bytes()[0] != b'.'
                        && let Ok(inner_iter) = inner(&path)
                    {
                        inner_iter
                    } else {
                        Box::new(std::iter::empty())
                    }
                })
            }))),
            Err(e) => Err(e.into()),
        }
    }

    inner(&path.to_path_buf())
}
