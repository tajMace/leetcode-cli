// entrypoint — parse CLI args, load config, dispatch to commands::{pull,test,submit}

mod cli;
use clap::Parser;
use cli::Cli;

mod client;
mod commands;
mod config;
mod error;
mod manifest;
mod models;

fn main() {
    let cli = Cli::parse();

    let result = match cli.command {
        cli::Command::Pull { slug, lang } => commands::pull(slug, lang),
        cli::Command::Test { slug } => Ok(println!("{slug}")),
        cli::Command::Submit { slug } => Ok(println!("{slug}")),
    };

    if let Err(e) = result {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}
