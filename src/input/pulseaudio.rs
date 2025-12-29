use byteorder::ReadBytesExt;
use pulseaudio::protocol::{self, SourceInfo};
use ringbuf::traits::Producer;
use std::fmt::Display;
use std::io::BufReader;
use std::os::unix::net::UnixStream;
use std::{ffi::CString, io::Read};

/// A simple PulseAudio client for connecting to the PulseAudio server
/// and recording audio from monitor sources.
#[derive(Debug)]
pub struct Client {
    sock: BufReader<UnixStream>,
    protocol_version: u16,
    sequce_number: u32,
}

impl Client {
    /// Connect to the PulseAudio server and initialize the client.
    pub fn connect() -> Result<Self, PulseAudioError> {
        let (sock, protocol_version, sequence_number) = connect_and_init()?;
        Ok(Client {
            sock,
            protocol_version,
            sequce_number: sequence_number,
        })
    }

    /// Send a command to the PulseAudio server and receive a reply.
    pub fn command<T: protocol::CommandReply>(
        &mut self,
        cmd: &protocol::Command,
    ) -> Result<T, PulseAudioError> {
        command(
            &mut self.sock,
            self.protocol_version,
            &mut self.sequce_number,
            cmd,
        )
    }

    /// Get a list of monitor sources available on the PulseAudio server.
    pub fn get_monitors(&mut self) -> Result<Vec<SourceInfo>, PulseAudioError> {
        let source_list: Vec<protocol::SourceInfo> =
            self.command(&protocol::Command::GetSourceInfoList)?;

        let sources = source_list
            .into_iter()
            .filter(|info| {
                info.name
                    .clone()
                    .into_string()
                    .unwrap()
                    .ends_with(".monitor")
            })
            .collect::<Vec<_>>();

        Ok(sources)
    }

    /// Start recording audio from the specified monitor source.
    ///
    /// The recorded audio samples will be pushed into the provided ring buffer producer.
    /// The `latency` parameter specifies the desired latency in bytes.
    /// Returns a handle to the recording thread.
    pub fn record_from_source(
        mut self,
        source_info: &SourceInfo,
        latency: u32,
        mut prod: impl Producer<Item = f32> + Send + 'static,
    ) -> Result<std::thread::JoinHandle<()>, PulseAudioError> {
        let params = protocol::RecordStreamParams {
            source_name: Some(source_info.name.clone()),
            sample_spec: protocol::SampleSpec {
                format: source_info.sample_spec.format,
                channels: source_info.channel_map.num_channels(),
                sample_rate: source_info.sample_spec.sample_rate,
            },
            channel_map: source_info.channel_map,
            cvolume: Some(protocol::ChannelVolume::norm(2)),
            buffer_attr: protocol::stream::BufferAttr {
                fragment_size: latency,
                max_length: latency * 4,
                target_length: latency,
                pre_buffering: 0,
                minimum_request_length: latency,
            },
            ..Default::default()
        };
        let record_stream = self.command::<protocol::CreateRecordStreamReply>(
            &protocol::Command::CreateRecordStream(params),
        )?;

        let handle = std::thread::spawn(move || {
            let mut buf = vec![0; record_stream.buffer_attr.fragment_size as usize];
            loop {
                let Ok(desc) = protocol::read_descriptor(&mut self.sock) else {
                    continue;
                };

                // A channel of -1 is a command message. Everything else is data.
                if desc.channel == u32::MAX {
                    continue;
                };

                buf.resize(desc.length as usize, 0);
                let Ok(()) = self.sock.read_exact(&mut buf) else {
                    continue;
                };

                let mut cursor = std::io::Cursor::new(buf.as_slice());
                while cursor.position() < cursor.get_ref().len() as u64 {
                    match record_stream.sample_spec.format {
                        protocol::SampleFormat::S16Le => {
                            let sample = cursor.read_i16::<byteorder::LittleEndian>().unwrap();
                            let _ = prod.try_push(sample as f32 / i16::MAX as f32);
                        }
                        protocol::SampleFormat::Float32Le => {
                            let sample = cursor.read_f32::<byteorder::LittleEndian>().unwrap();
                            let _ = prod.try_push(sample);
                        }
                        protocol::SampleFormat::S32Le => {
                            let sample = cursor.read_i32::<byteorder::LittleEndian>().unwrap();
                            let _ = prod.try_push(sample as f32 / i32::MAX as f32);
                        }
                        _ => unreachable!(),
                    };
                }
            }
        });

        Ok(handle)
    }
}

fn command<T: protocol::CommandReply>(
    sock: &mut BufReader<UnixStream>,
    protocol_version: u16,
    sequce_number: &mut u32,
    command: &protocol::Command,
) -> Result<T, PulseAudioError> {
    protocol::write_command_message(
        sock.get_mut(),
        *sequce_number,
        &command.clone(),
        protocol_version,
    )
    .map_err(|e| PulseAudioError::Command {
        sent: command.clone(),
        received: e,
    })?;
    *sequce_number += 1;

    protocol::read_reply_message::<T>(sock, protocol_version)
        .map_err(|e| PulseAudioError::Command {
            sent: command.clone(),
            received: e,
        })
        .map(|(_, reply)| reply)
}

fn connect_and_init() -> Result<(BufReader<UnixStream>, u16, u32), PulseAudioError> {
    let socket_path = pulseaudio::socket_path_from_env().ok_or(PulseAudioError::Connection)?;
    let mut sock = std::io::BufReader::new(
        UnixStream::connect(socket_path).map_err(|_| PulseAudioError::Connection)?,
    );

    let cookie = pulseaudio::cookie_path_from_env()
        .and_then(|path| std::fs::read(path).ok())
        .unwrap_or_default();
    let auth = protocol::AuthParams {
        version: protocol::MAX_VERSION,
        supports_shm: false,
        supports_memfd: false,
        cookie,
    };
    let mut seq = 0;
    let auth_reply = command::<protocol::AuthReply>(
        &mut sock,
        protocol::MAX_VERSION,
        &mut seq,
        &protocol::Command::Auth(auth.clone()),
    )?;
    let protocol_version = std::cmp::min(protocol::MAX_VERSION, auth_reply.version);

    let mut props = protocol::Props::new();
    props.set(
        protocol::Prop::ApplicationName,
        CString::new("volcano-rs").unwrap(),
    );
    let _ = command::<protocol::SetClientNameReply>(
        &mut sock,
        protocol_version,
        &mut seq,
        &protocol::Command::SetClientName(props),
    )?;
    Ok((sock, protocol_version, seq))
}

/// An error that can occur when interacting with the PulseAudio server.
#[derive(Debug)]
pub enum PulseAudioError {
    Connection,

    Command {
        sent: protocol::Command,
        received: protocol::ProtocolError,
    },
}

impl std::error::Error for PulseAudioError {}

impl Display for PulseAudioError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PulseAudioError::Connection => write!(f, "Failed to connect to PulseAudio server"),
            PulseAudioError::Command { sent, received } => write!(
                f,
                "Error executing PulseAudio command {:?}: {:?}",
                sent, received
            ),
        }
    }
}
