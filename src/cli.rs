// clap arg definitions: `pull <slug>`, `test <slug>`, `submit <slug>`

use clap::{Parser, Subcommand};

#[derive(Debug, Subcommand)]
pub enum Command {
    Pull { slug: String },
    Test { slug: String },
    Submit { slug: String },
}

#[derive(Debug, Parser)]
#[command(
    name = "leetcode-cli",
    about = "Pull, test, and submit LeetCode problems from the terminal"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}
