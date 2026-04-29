use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

use crossbeam::channel::{Receiver, Sender, unbounded};
use rodio::{Decoder, MixerDeviceSink, Player, Source};

pub const SUPPORTED_FORMATS: &[&str] = &["m4a", "mp3"];

pub enum AudioCommand {
    Play(PathBuf),
    TogglePause,
    Omit,
    RequestDuration,
    RequestPosition,
}

pub enum AudioEvent {
    PlaybackEnded,
    Duration(Option<Duration>),
    Position(Option<Duration>),
    Paused(Option<bool>),
}

struct Playback {
    player: Player,
    duration: Duration,
}

pub struct AudioController {
    cmd_tx: Sender<AudioCommand>,
    pub event_rx: Receiver<AudioEvent>,
}

impl Default for AudioController {
    fn default() -> Self {
        let (cmd_tx, cmd_rx) = unbounded::<AudioCommand>();
        let (event_tx, event_rx) = unbounded::<AudioEvent>();

        thread::spawn(move || {
            audio_thread(cmd_rx, event_tx);
        });

        AudioController { cmd_tx, event_rx }
    }
}

impl AudioController {
    pub fn play(&self, path: &Path) {
        let _ = self.cmd_tx.send(AudioCommand::Play(path.to_path_buf()));
    }

    pub fn toggle_pause(&self) {
        let _ = self.cmd_tx.send(AudioCommand::TogglePause);
    }

    pub fn omit(&self) {
        let _ = self.cmd_tx.send(AudioCommand::Omit);
    }

    pub fn request_duration(&self) {
        let _ = self.cmd_tx.send(AudioCommand::RequestDuration);
    }

    pub fn request_position(&self) {
        let _ = self.cmd_tx.send(AudioCommand::RequestPosition);
    }
}

fn audio_thread(cmd_rx: Receiver<AudioCommand>, event_tx: Sender<AudioEvent>) {
    let mut current_playback: Option<Playback> = None;
    let handle = rodio::DeviceSinkBuilder::open_default_sink().unwrap();

    loop {
        match cmd_rx.recv_timeout(Duration::from_millis(100)) {
            Ok(cmd) => {
                use AudioCommand::*;
                match cmd {
                    Play(path) => {
                        if let Some(pb) = current_playback.take() {
                            pb.player.stop();
                        }

                        if let Ok(playback) = play_track(&handle, path) {
                            current_playback = Some(playback);
                        }
                    }
                    TogglePause => {
                        let paused = if let Some(pb) = &mut current_playback {
                            if pb.player.is_paused() {
                                pb.player.play();
                                Some(false)
                            } else {
                                pb.player.pause();
                                Some(true)
                            }
                        } else {
                            None
                        };

                        let _ = event_tx.send(AudioEvent::Paused(paused));
                    }
                    Omit => {
                        if let Some(pb) = &current_playback {
                            pb.player.clear();
                        }
                        current_playback = None;
                    }
                    RequestDuration => {
                        let duration_opt = if let Some(pb) = &current_playback {
                            Some(pb.duration)
                        } else {
                            None
                        };

                        let _ = event_tx.send(AudioEvent::Duration(duration_opt));
                    }
                    RequestPosition => {
                        let position_opt = if let Some(pb) = &current_playback {
                            Some(pb.player.get_pos())
                        } else {
                            None
                        };

                        let _ = event_tx.send(AudioEvent::Position(position_opt));
                    }
                }
            }
            Err(_) => {
                if let Some(pb) = &current_playback
                    && pb.player.empty()
                {
                    current_playback = None;
                    let _ = event_tx.send(AudioEvent::PlaybackEnded);
                }
            }
        }
    }
}

fn play_track(handle: &MixerDeviceSink, path: PathBuf) -> anyhow::Result<Playback> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);
    // buffer source to prevent underflows
    let source = Decoder::new(reader)?.buffered();

    let duration = source.total_duration().unwrap_or_default();

    let player = Player::connect_new(handle.mixer());
    player.append(source);

    Ok(Playback { player, duration })
}
