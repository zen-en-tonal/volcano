//! Audio visualiser module.
//! Provides functionality to visualise audio input from various sources.
//! Includes different formatters and input handling.
//!
//! Modules:
//! - cava: Integration with the Cava audio visualisation library.
//! - input: Handling different audio input sources.
//! - output: Different formatters for visualising audio levels.

mod cava;
mod input;
mod output;

use std::fmt::Display;
use std::io::{self, Write};
use std::thread::{JoinHandle, sleep};
use std::time::Duration;

use cava::{Cava, CavaError};
pub use input::Inputs;
use input::pulseaudio::SourceInfo;
pub use output::{AsciiFormatter, Channel, DotFormatter, WaybarFormatter};
use ringbuf::HeapRb;
use ringbuf::traits::{Consumer, Split};

/// Configuration for the audio visualiser.
#[derive(Debug)]
pub struct Visualiser<T> {
    pub bars: usize,
    pub auto_sensitivity: bool,
    pub noise_reduction: f32,
    pub lowcut: u32,
    pub highcut: u32,
    pub fps: u32,
    pub latency: u32,
    pub threshold: f32,
    pub channel: Channel,
    pub input: input::Inputs,
    pub formatter: T,
}

impl Default for Visualiser<AsciiFormatter> {
    /// Create a default configuration for the visualiser.
    /// - bars: 40
    /// - auto_sensitivity: true
    /// - noise_reduction: 0.77
    /// - lowcut: 80
    /// - highcut: 16000
    /// - fps: 60
    /// - latency: 256
    /// - threshold: -20.0
    /// - channel: Stereo
    /// - formatter: AsciiFormatter with max_level 1000 and separator ";"
    fn default() -> Self {
        Visualiser {
            bars: 40,
            auto_sensitivity: true,
            noise_reduction: 0.77,
            lowcut: 80,
            highcut: 16000,
            fps: 60,
            latency: 256,
            threshold: -20.0,
            channel: Channel::Stereo,
            input: input::Inputs::pulseaudio(|_| true, 256).unwrap(),
            formatter: AsciiFormatter {
                max_level: 1000,
                separator: ";".to_string(),
            },
        }
    }
}

impl<T: Formatter> Visualiser<T> {
    /// Start the visualiser with the given monitor selector function.
    pub fn start(self) -> Result<JoinHandle<()>, VisualiserError> {
        let frame_size = self.input.frame_size(self.fps);

        let rb = HeapRb::<f32>::new((frame_size * 4) as usize);
        let (producer, mut consumer) = rb.split();

        // Determine the number of bars based on the selected channel mode.
        let bars = match self.channel {
            Channel::Left | Channel::Right | Channel::Average => self.bars * 2,
            Channel::Stereo => self.bars,
        };

        let sample_rate = self.input.sample_rate();
        let channels = self.input.channels();

        let record_handle = self.input.start_recording(producer)?;

        let output_handle = std::thread::spawn(move || {
            let mut buffer = vec![0f32; frame_size as usize];
            let mut cava_out = vec![0f64; self.bars as usize * channels as usize];

            let cava = Cava::new(
                bars as i32,
                sample_rate,
                channels as i32,
                self.auto_sensitivity as i32,
                self.noise_reduction as f64,
                self.lowcut as i32,
                self.highcut as i32,
            )
            .unwrap();

            let out = io::stdout();
            let mut out = out.lock();

            let sleep_duration = Duration::new(0, 1_000_000_000u32 / self.fps as u32);

            loop {
                sleep(sleep_duration);

                let _ = consumer.pop_slice(&mut buffer);
                cava.execute(&mut buffer, &mut cava_out);

                let levels = self.channel.levels(
                    &mut cava_out,
                    self.formatter.max_level(),
                    self.threshold as f64,
                );
                let formatted_levels = self.formatter.format(&levels);

                writeln!(out, "{}", formatted_levels).unwrap();
            }
        });

        let handle = std::thread::spawn(move || {
            record_handle.join().unwrap();
            output_handle.join().unwrap();
        });

        Ok(handle)
    }
}

/// Trait for selecting a monitor source.
pub trait MonitorSelector {
    /// Determine if the given monitor source should be selected.
    fn select(&self, info: &SourceInfo) -> bool;
}

impl<T> MonitorSelector for T
where
    T: Fn(&SourceInfo) -> bool,
{
    fn select(&self, info: &SourceInfo) -> bool {
        self(info)
    }
}

/// Predefined monitor selection strategies.
#[derive(Debug, Clone)]
pub enum MonitorSelection {
    /// Select the first available monitor.
    First,
    /// Select a monitor by its name.
    ByName(String),
}

impl MonitorSelector for MonitorSelection {
    fn select(&self, info: &SourceInfo) -> bool {
        match self {
            MonitorSelection::First => true,
            MonitorSelection::ByName(name) => &info.name.to_str().unwrap_or("") == name,
        }
    }
}

/// Trait for formatting audio levels into a string representation.
pub trait Formatter: Send + 'static {
    /// Get the maximum level for the formatter.
    fn max_level(&self) -> u32;
    /// Format the given levels into a string.
    fn format(&self, levels: &[u32]) -> String;
}

#[derive(Debug)]
pub enum VisualiserError {
    Cava(CavaError),
    Record(input::RecorderError),
}

impl From<CavaError> for VisualiserError {
    fn from(err: CavaError) -> Self {
        VisualiserError::Cava(err)
    }
}

impl From<input::RecorderError> for VisualiserError {
    fn from(err: input::RecorderError) -> Self {
        VisualiserError::Record(err)
    }
}

impl Display for VisualiserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VisualiserError::Cava(e) => write!(f, "Cava error: {}", e),
            VisualiserError::Record(e) => write!(f, "Input error: {}", e),
        }
    }
}

impl std::error::Error for VisualiserError {}
