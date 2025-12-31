use clap::Parser;

use crate::cmd::Commands;

mod cmd;
mod player;
mod visualiser;

fn main() {
    let cli = cmd::Cli::parse();

    match cli.command {
        Commands::Visualise(args) => {
            cmd::visualise::start_visualiser(args);
        }
        Commands::TogglePlayPause => {
            cmd::playctl::toggle_play_pause();
        }
        Commands::Next => {
            cmd::playctl::next();
        }
        Commands::Previous => {
            cmd::playctl::previous();
        }
    }
}
