use mpris::PlayerFinder;
use std::{fmt::Display, sync::mpsc, time::Duration};

/// Server to interact with media players via MPRIS.
#[derive(Debug, Clone)]
pub struct PlayerServer {
    pub cache_ttl: std::time::Duration,
    cmd: mpsc::Sender<Command>,
}

impl PlayerServer {
    /// Start the player server with a specified cache TTL.
    pub fn start(cache_ttl: std::time::Duration) -> (Self, std::thread::JoinHandle<()>) {
        let (cmd_tx, cmd_rx) = mpsc::channel::<Command>();

        let handle = std::thread::spawn(move || {
            let finder = PlayerFinder::new().unwrap();

            let mut info = None;
            let mut last_fetched = std::time::Instant::now() - cache_ttl;

            loop {
                while let Ok(command) = cmd_rx.recv() {
                    match command {
                        Command::GetInfo { respond_to } => {
                            // Check cache first and return if still valid
                            if last_fetched + cache_ttl > std::time::Instant::now() {
                                let _ = respond_to.send(info.clone());
                                continue;
                            }

                            // Get the active player
                            let player = match finder.find_active() {
                                Ok(p) => p,
                                Err(_) => {
                                    let _ = respond_to.send(None);
                                    continue;
                                }
                            };

                            // Fetch latest info
                            let latest_info = match get_info(&player) {
                                Some(info) => Some(info),
                                None => None,
                            };
                            let _ = respond_to.send(latest_info.clone());
                            last_fetched = std::time::Instant::now();
                            info = latest_info;
                        }
                        Command::TogglePlayPause => {
                            if let Ok(player) = finder.find_active() {
                                let _ = player.play_pause();
                            }
                        }
                        Command::Next => {
                            if let Ok(player) = finder.find_active() {
                                let _ = player.next();
                            }
                        }
                        Command::Previous => {
                            if let Ok(player) = finder.find_active() {
                                let _ = player.previous();
                            }
                        }
                    }
                }
            }
        });

        (
            Self {
                cache_ttl,
                cmd: cmd_tx,
            },
            handle,
        )
    }

    /// Get the current playing information.
    pub fn get_info(&self) -> Option<PlayingInfo> {
        let (resp_tx, resp_rx) = mpsc::channel::<Option<PlayingInfo>>();
        let command = Command::GetInfo {
            respond_to: resp_tx,
        };
        if self.cmd.send(command).is_err() {
            return None;
        }
        resp_rx.recv().ok().flatten()
    }

    /// Toggle play/pause state.
    pub fn toggle_play_pause(&self) {
        let command = Command::TogglePlayPause;
        let _ = self.cmd.send(command);
    }

    /// Skip to the next track.
    pub fn next(&self) {
        let command = Command::Next;
        let _ = self.cmd.send(command);
    }

    /// Return to the previous track.
    pub fn previous(&self) {
        let command = Command::Previous;
        let _ = self.cmd.send(command);
    }
}

fn get_info(player: &mpris::Player) -> Option<PlayingInfo> {
    let metadata = player.get_metadata().ok()?;
    let title = metadata.title().unwrap_or("Unknown Title").to_string();
    let artist = metadata
        .artists()
        .and_then(|artists| Some(artists.join(", ")))
        .unwrap_or("Unknown Artist".to_string());
    let length = metadata.length().unwrap_or(Duration::ZERO).as_secs_f32();
    let position = player.get_position().ok()?.as_secs_f32();
    let state = format!(
        "{:?}",
        player
            .get_playback_status()
            .unwrap_or(mpris::PlaybackStatus::Stopped)
    );

    Some(PlayingInfo {
        position,
        length,
        state,
        title,
        artist,
    })
}

enum Command {
    GetInfo {
        respond_to: mpsc::Sender<Option<PlayingInfo>>,
    },
    TogglePlayPause,
    Next,
    Previous,
}

#[derive(Debug, Clone)]
pub struct PlayingInfo {
    pub position: f32,
    pub length: f32,
    pub state: String,
    pub title: String,
    pub artist: String,
}

impl PlayingInfo {
    /// Get the progress of the current track as a float between 0.0 and 1.0.
    pub fn progress(&self) -> f32 {
        if self.length == 0.0 {
            0.0
        } else {
            self.position / self.length
        }
    }
}

impl Display for PlayingInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} - {} [{} / {}]",
            self.artist,
            self.title,
            format_duration(self.position),
            format_duration(self.length),
        )
    }
}

fn format_duration(seconds: f32) -> String {
    let total_seconds = seconds as u32;
    let minutes = total_seconds / 60;
    let seconds = total_seconds % 60;
    format!("{:02}:{:02}", minutes, seconds)
}
