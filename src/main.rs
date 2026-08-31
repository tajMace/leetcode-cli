// entrypoint — parse CLI args, load config, dispatch to commands::{pull,test,submit}

mod cli;
use clap::Parser;
use cli::Cli;

mod client;
mod commands;
mod config;
mod error;
mod models;

fn main() {
    let cli = Cli::parse();

    println!("{:?}", &cli);
}
