mod visualiser;

use clap::Parser;
use visualiser::*;

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
            max_level: METERS.len() as u32 - 1,
            threshold: args.threshold,
        }
    }
}

fn main() {
    let args = Args::parse();
    let visualiser: Visualiser = args.into();
    let handle = visualiser.start(select_first_monitor, dots).unwrap();

    handle.join().unwrap();
}

const METERS: [[char; 5]; 5] = [
    ['⠀', '⢀', '⢠', '⢰', '⢸'],
    ['⡀', '⣀', '⣠', '⣰', '⣸'],
    ['⡄', '⣄', '⣤', '⣴', '⣼'],
    ['⡆', '⣆', '⣦', '⣶', '⣾'],
    ['⡇', '⣇', '⣧', '⣷', '⣿'],
];

fn dots(levels: &[u32]) -> String {
    levels
        .chunks(2)
        .map(|chunk| {
            let left = chunk[0] as usize;
            let right = if chunk.len() > 1 {
                chunk[1] as usize
            } else {
                0
            };
            METERS[left][right]
        })
        .collect()
}
