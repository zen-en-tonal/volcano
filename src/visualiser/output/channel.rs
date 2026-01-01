use std::fmt::Display;

fn linear_to_db(linear: f64) -> f64 {
    20.0 * linear.log10()
}

/// Strategy for processing audio input levels.
#[derive(Debug, Clone)]
pub enum Channel {
    /// Average the left and right channels.
    Average,
    /// Use the left channel only.
    Left,
    /// Use the right channel only.
    Right,
    /// Use both channels in stereo.
    Stereo,
}

impl Display for Channel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Channel::Average => "Average",
            Channel::Left => "Left",
            Channel::Right => "Right",
            Channel::Stereo => "Stereo",
        };
        write!(f, "{}", s)
    }
}

impl From<&str> for Channel {
    fn from(s: &str) -> Self {
        match s.to_lowercase().as_str() {
            "average" => Channel::Average,
            "left" => Channel::Left,
            "right" => Channel::Right,
            "stereo" => Channel::Stereo,
            _ => Channel::Stereo,
        }
    }
}

impl Channel {
    fn process(&self, input: &mut [f64]) -> Vec<f64> {
        match self {
            Channel::Average => {
                let (l, r) = input.split_at_mut(input.len() / 2);
                l.iter().zip(r.iter()).map(|(r, l)| (r + l) / 2.0).collect()
            }
            Channel::Left => {
                let (l, _) = input.split_at_mut(input.len() / 2);
                l.to_vec()
            }
            Channel::Right => {
                let (_, r) = input.split_at_mut(input.len() / 2);
                r.to_vec()
            }
            Channel::Stereo => {
                let (l, r) = input.split_at_mut(input.len() / 2);
                r.reverse();
                l.iter().chain(r.iter()).cloned().collect()
            }
        }
    }

    /// Convert the processed input levels to dB levels.
    pub fn levels(&self, input: &mut [f64], max: u32, th: f64) -> Vec<u32> {
        let processed = self.process(input);
        let mut output = Vec::with_capacity(processed.len());
        for i in processed {
            let db = linear_to_db(i);
            let index = if db <= th {
                0
            } else if db >= 0.0 {
                max
            } else {
                ((db + th.abs()) / th.abs() * max as f64).round() as u32
            };
            output.push(index);
        }
        output
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_levels() {
        let mut input = vec![2.0, 0.32, 0.1, 0.032, 0.01, 0.001];
        let max = 10;
        let result = Channel::Stereo.levels(&mut input, max, -60.0);
        assert_eq!(result, vec![10, 8, 7, 0, 3, 5]);
    }
}
