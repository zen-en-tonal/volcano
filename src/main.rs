mod visualiser;

use std::fmt::Display;

use clap::Parser;
use visualiser::*;

const FETCH_INFO_INTERVAL: std::time::Duration = std::time::Duration::from_millis(200);

/// Command line arguments for the visualiser.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Number of bars to display
    #[arg(long, default_value_t = 40)]
    bars: usize,

    /// Auto sensitivity
    #[arg(long, default_value_t = true)]
    auto_sensitivity: bool,

    /// Noise reduction level
    #[arg(long, default_value_t = 0.77)]
    noise_reduction: f32,

    /// Low cut-off frequency
    #[arg(long, default_value_t = 80)]
    lowcut: u32,

    /// High cut-off frequency
    #[arg(long, default_value_t = 16000)]
    highcut: u32,

    /// Frames per second
    #[arg(long, default_value_t = 60)]
    fps: u32,

    /// Latency in samples
    #[arg(long, default_value_t = 256)]
    latency: u32,

    /// Threshold in dB
    #[arg(long, default_value_t = -20.0)]
    threshold: f32,
}

impl From<Args> for Visualiser {
    fn from(args: Args) -> Self {
        Visualiser {
            bars: args.bars,
            auto_sensitivity: args.auto_sensitivity,
            noise_reduction: args.noise_reduction,
            lowcut: args.lowcut,
            highcut: args.highcut,
            fps: args.fps,
            latency: args.latency,
            max_level: METERS[0][0].len() as u32 - 1,
            threshold: args.threshold,
        }
    }
}

#[derive(Debug, Clone)]
struct PlayingInfo {
    position: f32,
    length: f32,
    state: String,
    title: String,
    artist: String,
}

impl PlayingInfo {
    fn progress(&self) -> f32 {
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

fn get_info() -> Option<PlayingInfo> {
    let mut get_stats = std::process::Command::new("playerctl");
    let get_stats = get_stats.args([
        "metadata",
        "--format",
        "{{ position/1000000 }},{{ mpris:length/1000000 }},{{ status }},{{ title }},{{ artist }}",
    ]);

    let output = get_stats.output().ok()?;
    let stats = String::from_utf8(output.stdout).ok()?;
    let parts: Vec<&str> = stats.trim().split(',').collect();
    if parts.len() != 5 {
        return None;
    }
    let position = parts[0].parse::<f32>().ok()?;
    let length = parts[1].parse::<f32>().ok()?;
    let state = parts[2].to_string();
    let title = parts[3].to_string();
    let artist = parts[4].to_string();

    Some(PlayingInfo {
        position,
        length,
        state,
        title,
        artist,
    })
}

struct GetInfo;

fn main() {
    let args = Args::parse();
    let visualiser: Visualiser = args.into();

    let (get_tx, get_rx) = std::sync::mpsc::channel::<GetInfo>();
    let (tx, rx) = std::sync::mpsc::channel::<Option<PlayingInfo>>();

    let stats_handle = std::thread::spawn(move || {
        let mut prev_info = get_info();
        let mut last = std::time::Instant::now();

        loop {
            get_rx.recv().unwrap();

            if last + FETCH_INFO_INTERVAL > std::time::Instant::now() {
                tx.send(prev_info.clone()).unwrap();
                continue;
            }

            let info = match get_info() {
                Some(info) => {
                    prev_info = Some(info.clone());
                    Some(info)
                }
                None => prev_info.clone(),
            };

            tx.send(info).unwrap();
            last = std::time::Instant::now();
        }
    });

    let vis_handle = visualiser
        .start(select_first_monitor, move |levels| {
            get_tx.send(GetInfo).unwrap();
            let info = rx.try_recv().ok().flatten();
            let pos_rate = info.as_ref().map_or(0.0, |i| i.progress());
            let dots_str = dots(&levels, pos_rate);
            waybar(&dots_str, info)
        })
        .unwrap();

    vis_handle.join().unwrap();
    stats_handle.join().unwrap();
}

const METERS: [[[char; 5]; 5]; 3] = [
    [
        ['⠀', '⢀', '⢠', '⢰', '⢸'],
        ['⡀', '⣀', '⣠', '⣰', '⣸'],
        ['⡄', '⣄', '⣤', '⣴', '⣼'],
        ['⡆', '⣆', '⣦', '⣶', '⣾'],
        ['⡇', '⣇', '⣧', '⣷', '⣿'],
    ],
    [
        ['⠀', '⢀', '⢠', '⢰', '⢸'],
        ['⠀', '⢀', '⢠', '⢰', '⢸'],
        ['⠄', '⢄', '⢤', '⢴', '⢼'],
        ['⠆', '⢆', '⢦', '⢶', '⢾'],
        ['⠇', '⢇', '⢧', '⢷', '⢿'],
    ],
    [
        ['⠀', '⠀', '⠠', '⠰', '⠸'],
        ['⡀', '⡀', '⡠', '⡰', '⡸'],
        ['⡄', '⡄', '⡤', '⡴', '⡼'],
        ['⡆', '⡆', '⡦', '⡶', '⡾'],
        ['⡇', '⡇', '⡧', '⡷', '⡿'],
    ],
];

/// Formats levels into a string of dot characters.
fn dots(levels: &[u32], pos_rate: f32) -> String {
    let bars = levels.len() / 2;
    let current_index_in_bar = (bars as f32 * pos_rate).floor() as usize;
    let frac_pos = if (bars as f32 * pos_rate) - (current_index_in_bar as f32) < 0.5 {
        1
    } else {
        2
    };

    levels
        .chunks(2)
        .into_iter()
        .enumerate()
        .map(|(i, levels)| {
            let frac = if i == current_index_in_bar {
                frac_pos
            } else {
                0
            };
            let left_level = levels[0].max(1) as usize;
            let right_level = levels[1].max(1) as usize;

            METERS[frac][left_level][right_level]
        })
        .collect()
}

fn waybar(text: &str, info: Option<PlayingInfo>) -> String {
    match info {
        Some(info) => {
            let mut info_str = String::with_capacity(128);
            info_str.push_str("{\"text\":\"");
            info_str.push_str(text);
            info_str.push_str("\",\"tooltip\":\"");
            info_str.push_str(&info.to_string());
            info_str.push_str("\",\"class\":\"");
            info_str.push_str(&info.state.to_lowercase());
            info_str.push_str("\"}");

            info_str
        }
        None => {
            let mut info_str = String::with_capacity(64);
            info_str.push_str("{\"text\":\"");
            info_str.push_str(text);
            info_str.push_str("\",\"tooltip\":\"No player info available\",\"class\":\"stopped\"}");
            info_str
        }
    }
}
