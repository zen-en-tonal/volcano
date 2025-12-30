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
pub use output::Strategy;
use ringbuf::HeapRb;
use ringbuf::traits::{Consumer, Split};

/// Configuration for the audio visualiser.
#[derive(Debug, Clone)]
pub struct Visualiser {
    pub bars: usize,
    pub auto_sensitivity: bool,
    pub noise_reduction: f32,
    pub lowcut: u32,
    pub highcut: u32,
    pub fps: u32,
    pub latency: u32,
    pub max_level: u32,
    pub threshold: f32,
    pub strategy: Strategy,
}

impl Default for Visualiser {
    /// Create a default configuration for the visualiser.
    /// - bars: 40
    /// - auto_sensitivity: true
    /// - noise_reduction: 0.77
    /// - lowcut: 80
    /// - highcut: 16000
    /// - fps: 60
    /// - latency: 256
    /// - max_level: 100
    /// - threshold: -20.0
    /// - strategy: Stereo
    fn default() -> Self {
        Visualiser {
            bars: 40,
            auto_sensitivity: true,
            noise_reduction: 0.77,
            lowcut: 80,
            highcut: 16000,
            fps: 60,
            latency: 256,
            max_level: 100,
            threshold: -20.0,
            strategy: Strategy::Stereo,
        }
    }
}

impl Visualiser {
    /// Start the visualiser with the given monitor selector function.
    pub fn start(
        self,
        monitor_select: impl Fn(&SourceInfo) -> bool,
        formatter: impl Fn(&[u32]) -> String + Send + 'static,
    ) -> Result<JoinHandle<()>, VisualiserError> {
        let mut socket = Client::connect()?;
        let monitors = socket.get_monitors()?;
        let mon = monitors
            .into_iter()
            .find(|m| monitor_select(m))
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

                let levels =
                    self.strategy
                        .levels(&mut cava_out, self.max_level, self.threshold as f64);
                let formatted_levels = formatter(&levels);

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

/// Default monitor selector that selects first monitor.
pub fn select_first_monitor(_info: &SourceInfo) -> bool {
    true
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
