use ::pulseaudio::protocol::SourceInfo;
use ringbuf::traits::Producer;
use std::{error::Error, thread::JoinHandle};

use crate::visualiser::input::pulseaudio::Client;

pub mod pulseaudio;

/// Different audio input sources for the visualiser.
#[derive(Debug)]
pub enum Inputs {
    PulseAudio {
        source: SourceInfo,
        latency: u32,
        client: Client,
    },
}

impl Inputs {
    pub fn pulseaudio(
        monitor_select: impl Fn(&SourceInfo) -> bool,
        latency: u32,
    ) -> Result<Self, RecorderError> {
        let mut client = Client::connect()?;
        let monitors = client.get_monitors()?;
        let selected = monitors.iter().find(|info| monitor_select(info)).unwrap();

        Ok(Inputs::PulseAudio {
            source: selected.clone(),
            latency,
            client,
        })
    }

    /// Start recording audio from the selected input source.
    pub fn start_recording(
        self,
        producer: impl Producer<Item = f32> + Send + 'static,
    ) -> Result<JoinHandle<()>, RecorderError> {
        match self {
            Inputs::PulseAudio {
                source,
                latency,
                client,
            } => {
                let handle = client.record_from_source(&source, latency, producer)?;
                Ok(handle)
            }
        }
    }

    /// Calculate the frame size based on the input source and desired FPS.
    pub fn frame_size(&self, fps: u32) -> u32 {
        match self {
            Inputs::PulseAudio { source, .. } => {
                source.sample_spec.sample_rate / fps * source.channel_map.num_channels() as u32
            }
        }
    }

    /// Get the number of channels for the input source.
    pub fn channels(&self) -> u32 {
        match self {
            Inputs::PulseAudio { source, .. } => source.channel_map.num_channels() as u32,
        }
    }

    /// Get the sample rate for the input source.
    pub fn sample_rate(&self) -> u32 {
        match self {
            Inputs::PulseAudio { source, .. } => source.sample_spec.sample_rate,
        }
    }
}

/// Errors that can occur while setting up the audio input.
#[derive(Debug)]
pub enum RecorderError {
    PulseAudioError(pulseaudio::PulseAudioError),
}

impl From<pulseaudio::PulseAudioError> for RecorderError {
    fn from(err: pulseaudio::PulseAudioError) -> Self {
        RecorderError::PulseAudioError(err)
    }
}

impl Error for RecorderError {}

impl std::fmt::Display for RecorderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RecorderError::PulseAudioError(err) => write!(f, "PulseAudio error: {}", err),
        }
    }
}
