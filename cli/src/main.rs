#![warn(clippy::all, clippy::pedantic)]

use anyhow::Result;
use application::Cli;
use clap::{ArgAction, Parser};
use common::{error::FancyError, init::CloudInit};
use storage::Storage;

mod application;
mod storage;

// Include the build information generated by build.rs
include!(concat!(env!("OUT_DIR"), "/build_info.rs"));

pub const AUTHORS: [&str; 1] = ["HttpRafa"];

#[tokio::main]
async fn main() {
    async fn run() -> Result<()> {
        let args = Arguments::parse();
        CloudInit::init_logging(args.debug, true, Storage::latest_log_file());
        CloudInit::print_ascii_art("Atomic Cloud CLI", &VERSION, &AUTHORS);

        let mut cli = Cli::new().await?;
        cli.start().await
    }

    if let Err(error) = run().await {
        FancyError::print_fancy(&error, true);
    }
}

#[derive(Parser)]
pub struct Arguments {
    #[clap(short, long, help = "Enable debug mode", action = ArgAction::SetTrue)]
    pub debug: bool,
}
