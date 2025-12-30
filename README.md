# 🌋 Volcano

A terminal-based audio visualizer written in Rust that displays real-time audio frequency analysis using beautiful Unicode characters. Volcano leverages the powerful CAVA (Console-based Audio Visualizer for ALSA) library to provide smooth, responsive audio visualization.

## Features

- **Real-time Audio Visualization**: Displays audio frequency analysis as vertical bars using Unicode Braille patterns
- **PulseAudio Integration**: Seamlessly connects to PulseAudio for system audio capture
- **Waybar Integration**: Outputs formatted text suitable for use in Waybar status bars
- **Media Player Integration**: Shows current playing track information via `playerctl`
- **Highly Configurable**: Extensive command-line options for customization
- **Low Latency**: Optimized for real-time performance with configurable latency settings

## Prerequisites

- **Rust**: Latest stable version (2024 edition)
- **CAVA Library**: The libcava development library must be installed
- **PulseAudio**: For audio capture
- **playerctl** (optional): For media player integration

### Installing Dependencies

#### Ubuntu/Debian
```bash
sudo apt update
sudo apt install libcava-dev libpulse-dev playerctl
```

#### Arch Linux
```bash
sudo pacman -S cava pulseaudio playerctl
```

#### Fedora
```bash
sudo dnf install cava-devel pulseaudio-libs-devel playerctl
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

### Basic Usage
```bash
volcano
```

### Command Line Options

| Option | Default | Description |
|--------|---------|-------------|
| `--bars` | 40 | Number of frequency bars to display |
| `--auto-sensitivity` | true | Enable automatic sensitivity adjustment |
| `--noise-reduction` | 0.77 | Noise reduction level (0.0-1.0) |
| `--lowcut` | 80 | Low frequency cut-off (Hz) |
| `--highcut` | 16000 | High frequency cut-off (Hz) |
| `--fps` | 60 | Frames per second |
| `--latency` | 256 | Audio latency in samples |
| `--threshold` | -20.0 | Threshold level in dB |

### Examples

```bash
# High-resolution visualization with 80 bars
volcano --bars 80 --fps 120

# Lower sensitivity for quieter environments
volcano --threshold -30.0 --noise-reduction 0.9

# Optimize for bass-heavy music
volcano --lowcut 40 --highcut 8000
```

## Integration

### Waybar Configuration

Volcano is designed to work seamlessly with Waybar. Add this to your Waybar configuration:

```json
{
    "custom/volcano": {
        "exec": "volcano --bars 20 --fps 30",
        "format": "🎵 {}",
        "max-length": 50
    }
}
```

## Architecture

Volcano consists of several key components:

- **Audio Input**: PulseAudio integration for capturing system audio
- **FFT Processing**: CAVA library for frequency analysis
- **Visualization**: Unicode Braille patterns for terminal display
- **Output**: Configurable formatters (currently Waybar-focused)
- **Media Integration**: playerctl integration for track information

## Configuration

Volcano uses command-line arguments for configuration. For permanent settings, consider creating shell aliases:

```bash
# ~/.bashrc or ~/.zshrc
alias volcano-bass='volcano --lowcut 40 --highcut 200 --bars 20'
alias volcano-treble='volcano --lowcut 1000 --highcut 16000 --bars 30'
alias volcano-full='volcano --bars 80 --fps 120 --threshold -25'
```

## Troubleshooting

### Audio Not Detected
- Ensure PulseAudio is running: `pulseaudio --check -v`
- Check audio devices: `pactl list short sources`
- Verify permissions for audio access

### CAVA Library Issues
- Ensure libcava-dev is properly installed
- Check library path: `ldconfig -p | grep cava`
- Rebuild with: `cargo clean && cargo build`

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

- [CAVA](https://github.com/karlstav/cava) - The excellent audio analysis library
- [PulseAudio](https://www.freedesktop.org/wiki/Software/PulseAudio/) - Audio server
- [Waybar](https://github.com/Alexays/Waybar) - Wayland status bar

---

*Made with 🦀 and ❤️*