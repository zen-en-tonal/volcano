use crate::cmd::VisualiseArgs;
use crate::player;
use crate::visualiser::{Inputs, Visualiser, WaybarFormatter, DotFormatter};

impl From<VisualiseArgs> for Visualiser<WaybarFormatter<DotFormatter>> {
    fn from(args: VisualiseArgs) -> Self {
        Visualiser {
            bars: args.bars,
            auto_sensitivity: args.auto_sensitivity,
            noise_reduction: args.noise_reduction,
            lowcut: args.lowcut,
            highcut: args.highcut,
            fps: args.fps,
            latency: args.latency,
            threshold: args.threshold,
            channel: args.strategy,
            input: Inputs::pulseaudio(|_| true, args.latency).unwrap(),
            formatter: WaybarFormatter {
                player: None,
                inner: DotFormatter { player: None },
            },
        }
    }
}

pub fn start_visualiser(args: VisualiseArgs) {
    let mut visualiser: Visualiser<WaybarFormatter<DotFormatter>> = args.into();
    let (player, player_handle) = player::start_player(std::time::Duration::from_millis(200));
    visualiser.formatter.player = Some(player.clone());
    visualiser.formatter.inner.player = Some(player);

    let vis_handle = visualiser.start().unwrap();

    vis_handle.join().unwrap();
    player_handle.join().unwrap();
}
