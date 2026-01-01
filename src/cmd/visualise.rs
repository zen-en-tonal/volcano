use crate::cmd::VisualiseArgs;
use crate::player::{self, PlayingInfo};
use crate::visualiser::*;

impl From<VisualiseArgs> for Visualiser<MonitorSelection> {
    fn from(args: VisualiseArgs) -> Self {
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
            channel: args.strategy,
            monitor_select: MonitorSelection::First,
        }
    }
}

pub fn start_visualiser(args: VisualiseArgs) {
    let visualiser: Visualiser<_> = args.into();
    let (player, player_handle) =
        player::PlayerServer::start(std::time::Duration::from_millis(200));

    let vis_handle = visualiser
        .start(move |levels| {
            let info = player.get_info();
            let pos_rate = info.as_ref().map_or(0.0, |i| i.progress());
            let dots_str = dots(&levels, pos_rate);
            waybar(&dots_str, info)
        })
        .unwrap();

    vis_handle.join().unwrap();
    player_handle.join().unwrap();
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
