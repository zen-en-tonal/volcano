mod cava;
mod input;
mod output;

use std::fmt::Display;
use std::io;
use std::io::Write;
use std::thread::{JoinHandle, sleep};
use std::time::Duration;

use cava::{Cava, CavaError};
use input::pulseaudio::*;
pub use output::{AsciiFormatter, Channel, DotFormatter, WaybarFormatter};
use ringbuf::HeapRb;
use ringbuf::traits::{Consumer, Split};

/// Configuration for the audio visualiser.
#[derive(Debug, Clone)]
pub struct Visualiser<T, Q> {
    pub bars: usize,
    pub auto_sensitivity: bool,
    pub noise_reduction: f32,
    pub lowcut: u32,
    pub highcut: u32,
    pub fps: u32,
    pub latency: u32,
    pub threshold: f32,
    pub channel: Channel,
    pub monitor_select: T,
    pub formatter: Q,
}

impl Default for Visualiser<MonitorSelection, AsciiFormatter> {
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
    /// - monitor_select: First
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
            monitor_select: MonitorSelection::First,
            formatter: AsciiFormatter {
                max_level: 1000,
                separator: ";".to_string(),
            },
        }
    }
}

impl<T: MonitorSelector, Q: Formatter> Visualiser<T, Q> {
    /// Start the visualiser with the given monitor selector function.
    pub fn start(self) -> Result<JoinHandle<()>, VisualiserError> {
        let mut socket = Client::connect()?;
        let monitors = socket.get_monitors()?;
        let mon = monitors
            .into_iter()
            .find(|m| self.monitor_select.select(m))
            .ok_or(VisualiserError::NoMonitorFound)?;
        let frame_size =
            mon.sample_spec.sample_rate / self.fps * mon.channel_map.num_channels() as u32;

        let rb = HeapRb::<f32>::new((frame_size * 4) as usize);
        let (producer, mut consumer) = rb.split();

        let record_handle = socket
            .record_from_source(&mon, self.latency, producer)
            .unwrap();

        let output_handle = std::thread::spawn(move || {
            let mut buffer = vec![0f32; frame_size as usize];
            let mut cava_out =
                vec![0f64; self.bars as usize * mon.channel_map.num_channels() as usize];

            let cava = Cava::new(
                self.bars as i32,
                mon.sample_spec.sample_rate,
                mon.channel_map.num_channels() as i32,
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
    fn max_level(&self) -> u32;
    fn format(&self, levels: &[u32]) -> String;
}

#[derive(Debug)]
pub enum VisualiserError {
    CavaError(CavaError),
    PulseAudioError(PulseAudioError),
    NoMonitorFound,
}

impl From<CavaError> for VisualiserError {
    fn from(err: CavaError) -> Self {
        VisualiserError::CavaError(err)
    }
}

impl From<PulseAudioError> for VisualiserError {
    fn from(err: PulseAudioError) -> Self {
        VisualiserError::PulseAudioError(err)
    }
}

impl Display for VisualiserError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VisualiserError::CavaError(e) => write!(f, "Cava error: {}", e),
            VisualiserError::PulseAudioError(e) => write!(f, "PulseAudio error: {}", e),
            VisualiserError::NoMonitorFound => write!(f, "No monitor source found"),
        }
    }
}

impl std::error::Error for VisualiserError {}
