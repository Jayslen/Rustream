use clap::Parser;

mod actions;
mod database;
mod filesystem;
mod types;

use types::{Actions, Cli};

fn main() {
    let cli = Cli::parse();

    match &cli.action {
        Actions::Add => actions::add(&cli),
        Actions::Mod => {
            println!("Modifying a movie...");
        }
        Actions::Del => {
            println!("Deleting a movie...");
        }
    }
}
