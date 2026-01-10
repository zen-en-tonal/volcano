use crate::{player, visualiser::Formatter};

/// A simple ASCII formatter that joins levels with a separator.
#[derive(Debug, Clone)]
pub struct AsciiFormatter {
    pub max_level: u32,
    pub separator: String,
}

impl Formatter for AsciiFormatter {
    fn max_level(&self) -> u32 {
        self.max_level
    }

    fn format(&self, levels: &[u32]) -> String {
        levels
            .iter()
            .map(|level| level.to_string())
            .collect::<Vec<String>>()
            .join(&self.separator)
    }
}

/// A formatter that represents levels using dot characters.
#[derive(Debug, Clone)]
pub struct DotFormatter {
    pub player: Option<player::PlayerClient>,
}

impl Formatter for DotFormatter {
    fn max_level(&self) -> u32 {
        DOTS[0][0].len() as u32 - 1
    }

    fn format(&self, levels: &[u32]) -> String {
        let player = match &self.player {
            Some(p) => p,
            None => return dots(levels, 0.0),
        };
        let info = player.get_info();
        let pos_rate = info.as_ref().map_or(0.0, |i| i.progress());
        dots(&levels, pos_rate)
    }
}

/// A formatter that uses a template to format the output string.
#[derive(Debug, Clone)]
pub struct TemplateFormatter<T> {
    pub player: Option<player::PlayerClient>,
    pub inner: T,
    pub template: String,
}

impl<T: Formatter> Formatter for TemplateFormatter<T> {
    fn max_level(&self) -> u32 {
        self.inner.max_level()
    }

    fn format(&self, levels: &[u32]) -> String {
        let text = self.inner.format(levels);
        let info = match &self.player {
            Some(p) => p.get_info(),
            None => None,
        };
        match info {
            Some(info) => self
                .template
                .replace("{text}", &text)
                .replace("{position}", &info.position.to_string())
                .replace("{length}", &info.length.to_string())
                .replace("{title}", &info.title)
                .replace("{artist}", &info.artist)
                .replace("{state}", &info.state),
            None => self
                .template
                .replace("{text}", &text)
                .replace("{position}", "")
                .replace("{length}", "")
                .replace("{title}", "")
                .replace("{artist}", "")
                .replace("{state}", "stopped"),
        }
    }
}

pub const TEMPLATE_WAYBAR: &str =
    "{\"text\":\"{text}\",\"tooltip\":\"{artist} - {title}\",\"class\":\"{state}\"}";

const DOTS: [[[char; 5]; 5]; 3] = [
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

            DOTS[frac][left_level][right_level]
        })
        .collect()
}
