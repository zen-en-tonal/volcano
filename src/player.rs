use mpris::PlayerFinder;
use std::{fmt::Display, sync::mpsc, time::Duration};

/// Client to interact with media players via MPRIS.
/// The client communicates with a server running in its own thread via channels.
/// It supports fetching playing info and controlling playback.
///
/// This struct has only cloneable handles to the server, so cloning it is cheap.
#[derive(Debug, Clone)]
pub struct PlayerClient {
    cmd: mpsc::Sender<Command>,
}

/// Start the player server with a specified cache TTL.
pub fn start_player(cache_ttl: std::time::Duration) -> (PlayerClient, std::thread::JoinHandle<()>) {
    let (cmd_tx, cmd_rx) = mpsc::channel::<Command>();

    let handle = std::thread::spawn(move || {
        let finder = PlayerFinder::new().unwrap();

        let mut info = None;
        let mut last_fetched = std::time::Instant::now() - cache_ttl;

        loop {
            // Block until a command is received to avoid busy waiting
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
                    Command::TogglePlayPause { respond_to } => {
                        let res = match finder.find_active() {
                            Ok(p) => p.play_pause().is_ok(),
                            Err(_e) => false,
                        };
                        let _ = respond_to.send(res);
                    }
                    Command::Next { respond_to } => {
                        let res = match finder.find_active() {
                            Ok(p) => p.next().is_ok(),
                            Err(_e) => false,
                        };
                        let _ = respond_to.send(res);
                    }
                    Command::Previous { respond_to } => {
                        let res = match finder.find_active() {
                            Ok(p) => p.previous().is_ok(),
                            Err(_e) => false,
                        };
                        let _ = respond_to.send(res);
                    }
                }
            }
        }
    });

    (PlayerClient { cmd: cmd_tx }, handle)
}

impl PlayerClient {
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
    pub fn play_pause(&self) -> bool {
        let (resp_tx, resp_rx) = mpsc::channel::<bool>();
        let command = Command::TogglePlayPause {
            respond_to: resp_tx,
        };
        let _ = self.cmd.send(command);
        resp_rx.recv().unwrap_or(false)
    }

    /// Skip to the next track.
    pub fn next(&self) -> bool {
        let (resp_tx, resp_rx) = mpsc::channel::<bool>();
        let command = Command::Next {
            respond_to: resp_tx,
        };
        let _ = self.cmd.send(command);
        resp_rx.recv().unwrap_or(false)
    }

    /// Return to the previous track.
    pub fn previous(&self) -> bool {
        let (resp_tx, resp_rx) = mpsc::channel::<bool>();
        let command = Command::Previous {
            respond_to: resp_tx,
        };
        let _ = self.cmd.send(command);
        resp_rx.recv().unwrap_or(false)
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

#[derive(Debug, Clone)]
enum Command {
    GetInfo {
        respond_to: mpsc::Sender<Option<PlayingInfo>>,
    },
    TogglePlayPause {
        respond_to: mpsc::Sender<bool>,
    },
    Next {
        respond_to: mpsc::Sender<bool>,
    },
    Previous {
        respond_to: mpsc::Sender<bool>,
    },
}

/// Information about the currently playing track.
#[derive(Debug, Clone)]
pub struct PlayingInfo {
    /// Current position in seconds.
    pub position: f32,
    /// Total length in seconds.
    pub length: f32,
    /// Playback state (e.g., Playing, Paused).
    pub state: String,
    /// Title of the track.
    pub title: String,
    /// Artist of the track.
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
    let hours = minutes / 60;
    let mut str = String::with_capacity(16);
    if hours > 0 {
        str.push_str(&format!("{:02}:", hours));
    }
    str.push_str(&format!("{:02}:{:02}", minutes % 60, seconds));
    str
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_duration() {
        assert_eq!(format_duration(3661.0), "01:01:01");
        assert_eq!(format_duration(61.0), "01:01");
        assert_eq!(format_duration(59.0), "00:59");
    }
}
