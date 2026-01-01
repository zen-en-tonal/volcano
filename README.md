# 🌋 Volcano

A terminal-based audio visualiser written in Rust that displays real-time audio frequency analysis using Unicode characters. Volcano uses the CAVA library for smooth, responsive spectrum analysis, PulseAudio for system audio capture, and MPRIS (D-Bus) to optionally enrich output with current track information.

## Features

- **Real-time Visualisation**: Unicode Braille-based bar meter with track-progress hint
- **PulseAudio Support**: Captures system audio via monitor sources
- **Waybar Output**: Emits JSON formatted output suitable for Waybar `custom/*`
- **MPRIS Integration**: Optional player metadata via D-Bus (no external `playerctl` needed)
- **Configurable**: Tune bars, sensitivity, FPS, latency, thresholds, and channel strategy
- **Low Latency**: Optimised for real-time with adjustable buffer sizes

## Prerequisites

- **Rust**: Latest stable toolchain (Edition 2024)
- **CAVA**: `libcava` development library installed
- **PulseAudio**: For audio capture from monitor sources
- **MPRIS-compatible player** (optional): For metadata via D-Bus (e.g., VLC, Spotify, etc.)

### Installing Dependencies

#### Ubuntu/Debian
```bash
sudo apt update
sudo apt install libcava-dev libpulse-dev
```

#### Arch Linux
```bash
sudo pacman -S cava pulseaudio
```

#### Fedora
```bash
sudo dnf install cava-devel pulseaudio-libs-devel
```

## Installation

### From Source
```bash
git clone https://github.com/yourusername/volcano.git
cd volcano
cargo build --release
cargo install --path .
```

The binary will be installed to `~/.cargo/bin/volcano`.

## Usage

### CLI Overview
Volcano provides a single binary with subcommands:

```bash
volcano <SUBCOMMAND> [OPTIONS]
```

Subcommands:
- `visualise` — start the audio visualiser
- `play-pause` — toggle play/pause on the active MPRIS player
- `next` — skip to next track
- `previous` — go to previous track

### Basic Usage
```bash
volcano visualise
```

### Visualise Options

| Option | Default | Description |
|--------|---------|-------------|
| `--bars` | 20 | Number of frequency bars to display |
| `--auto-sensitivity` | true | Enable automatic sensitivity adjustment |
| `--noise-reduction` | 0.77 | Noise reduction level (0.0-1.0) |
| `--lowcut` | 80 | Low frequency cut-off (Hz) |
| `--highcut` | 16000 | High frequency cut-off (Hz) |
| `--fps` | 60 | Frames per second |
| `--latency` | 256 | Audio latency in samples |
| `--threshold` | -20.0 | Threshold level in dB |
| `--strategy` | Stereo | Channel strategy: `Average`, `Left`, `Right`, `Stereo` |

### Examples

```bash
# High-resolution visualisation with 80 bars
volcano visualise --bars 80 --fps 120

# Lower sensitivity for quieter environments
volcano visualise --threshold -30.0 --noise-reduction 0.9

# Optimise for bass-heavy music
volcano visualise --lowcut 40 --highcut 8000

# Control the player via MPRIS
volcano play-pause
volcano next
volcano previous
```

## Integration

### Waybar Configuration

Volcano emits JSON formatted output (text, tooltip, class). Add this to your Waybar configuration:

```json
{
    "custom/volcano": {
        "exec": "volcano visualise --bars 20 --fps 30",
        "return-type": "json",
        "max-length": 50
    }
}
```

## Architecture

Volcano is structured as a reusable library plus a CLI binary:

- **Library (`src/`)**
    - `lib.rs` — crate root exporting `player` and `visualiser`
    - `player.rs` — MPRIS client and server thread (get info, play/pause, next, previous)
    - `visualiser.rs` — orchestration of capture, CAVA processing, and output formatting
    - `visualiser/` — submodules:
        - `cava.rs` — FFI to CAVA via bindgen
        - `input.rs` + `input/pulseaudio.rs` — PulseAudio capture
        - `output.rs` — re-exports
        - `output/channel.rs` — channel strategies (Average, Left, Right, Stereo)
        - `output/formatter.rs` — `AsciiFormatter`, `DotFormatter`, `WaybarFormatter`

- **CLI (`bin/cli/`)**
    - `main.rs` — subcommand dispatch
    - `cmd.rs` — Clap definitions for `visualise`, `play-pause`, `next`, `previous`
    - `cmd/visualise.rs` — starts the visualiser using `WaybarFormatter<DotFormatter>`
    - `cmd/playctl.rs` — MPRIS playback controls

## Configuration

Volcano uses command-line arguments for configuration. For permanent settings, consider creating shell aliases:

```bash
# ~/.bashrc or ~/.zshrc
alias volcano-bass='volcano visualise --lowcut 40 --highcut 200 --bars 20'
alias volcano-treble='volcano visualise --lowcut 1000 --highcut 16000 --bars 30'
alias volcano-full='volcano visualise --bars 80 --fps 120 --threshold -25'
```

## Troubleshooting

### Audio Not Detected
- Ensure PulseAudio is running: `pulseaudio --check -v`
- Check audio devices: `pactl list short sources`
- Verify permissions for audio access

### CAVA Library Issues
- Ensure `libcava` development package is installed
- Check library path: `ldconfig -p | grep cava`
- Rebuild: `cargo clean && cargo build`

### High CPU Usage
- Reduce FPS: `--fps 30`
- Decrease bars: `--bars 20`
- Increase latency: `--latency 512`

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request. Areas where contributions would be particularly valuable:

- Additional audio backends (ALSA, JACK)
- More output formatters (tmux, i3blocks)
- Performance optimizations
- Cross-platform support
- Documentation improvements

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## Acknowledgments

- [CAVA](https://github.com/karlstav/cava) — audio analysis library
- [PulseAudio](https://www.freedesktop.org/wiki/Software/PulseAudio/) — audio server
- [Waybar](https://github.com/Alexays/Waybar) — Wayland status bar
- [MPRIS](https://specifications.freedesktop.org/mpris-spec/latest/) — media player interface over D-Bus

---

*Made with 🦀 and ❤️*