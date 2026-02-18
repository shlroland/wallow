use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "wallow")]
#[command(about = "A Rust CLI application framework", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    Hello {
        #[arg(short, long)]
        name: String,
    },
    Random,
}
