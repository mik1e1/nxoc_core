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

// pub enum AudioNodeType {
//     Folder,
//     AudioFile,
// }
//
// pub struct AudioNode {
//     path: PathBuf,
//     node_type: AudioNodeType,
//     children: Option<Vec<AudioNode>>,
// }
//
// pub fn produce_audio_node(path: &Path) -> anyhow::Result<AudioNode> {
//     let node_type_opt = if path.is_file()
//         && let Some(ext) = path.extension()
//         && SUPPORTED_FORMATS
//             .into_iter()
//             .any(|x| *x == ext.to_string_lossy())
//     {
//         Some(AudioNodeType::AudioFile)
//     } else if path.is_dir() {
//         Some(AudioNodeType::Folder)
//     } else {
//         None
//     };
//
//     if let Some(node_type) = node_type_opt {
//         let mut root = AudioNode {
//             path: path.to_path_buf(),
//             node_type,
//             children: None,
//         };
//
//         return Ok(root);
//     } else {
//         Err(anyhow!("Called function on miscellanious file"))
//     }
// }
//
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
