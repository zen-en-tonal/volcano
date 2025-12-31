pub mod playctl;
pub mod visualise;

use clap::{Parser, Subcommand};

use crate::visualiser::Strategy;

/// Command line arguments for the application.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    #[clap(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Start the audio visualiser
    Visualise(VisualiseArgs),
    TogglePlayPause,
    Next,
    Previous,
}

/// Command line arguments for the visualiser.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct VisualiseArgs {
    /// Number of bars to display
    #[arg(long, default_value_t = 20)]
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

    /// Strategy for processing audio input levels
    /// (Options: Average, Left, Right, Stereo)
    #[arg(long, default_value_t = Strategy::Stereo)]
    strategy: Strategy,
}
