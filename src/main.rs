use clap::Parser;

use crate::cmd::Commands;

pub mod cmd;
pub mod player;
pub mod visualiser;

fn main() {
    let cli = cmd::Cli::parse();

    match cli.command {
        Commands::Visualise(args) => {
            cmd::visualise::start_visualiser(args);
        }
        Commands::PlayPause => {
            cmd::playctl::play_pause();
        }
        Commands::Next => {
            cmd::playctl::next();
        }
        Commands::Previous => {
            cmd::playctl::previous();
        }
    }
}
