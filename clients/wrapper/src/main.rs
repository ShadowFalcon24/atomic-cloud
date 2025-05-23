#![warn(clippy::all, clippy::pedantic)]

use std::{path::PathBuf, process::exit};

use application::Wrapper;
use args::Args;
use clap::Parser;
use common::{error::FancyError, init::CloudInit};
use simplelog::info;

mod application;
mod args;

// Include the build information generated by build.rs
include!(concat!(env!("OUT_DIR"), "/build_info.rs"));

pub const AUTHORS: [&str; 1] = ["HttpRafa"];
pub const LOG_FILE: &str = "wrapper.log";

#[tokio::main(flavor = "multi_thread", worker_threads = 2)]
async fn main() {
    let args = Args::parse();
    CloudInit::init_logging(args.debug, false, PathBuf::from(LOG_FILE));
    CloudInit::print_ascii_art("Atomic Cloud Wrapper", &VERSION, &AUTHORS);

    info!("Starting wrapper...");
    let mut wrapper = Wrapper::new(args).await;
    if let Err(error) = wrapper.start().await {
        FancyError::print_fancy(&error, true);
    }
    info!("My work here is done. Terminating myself...");
    exit(0);
}
