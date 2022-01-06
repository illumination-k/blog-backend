mod markdown;
mod utils;
mod args;

use anyhow::Result;
use structopt::StructOpt;
use std::env::set_var;


use crate::args::{LogLevel, Opt, SubCommands};

fn main() -> Result<()> {
    let opt = Opt::from_args();

    match &opt.log_level {
        Some(log_level) => match log_level {
            LogLevel::DEBUG => set_var("RUST_LOG", "debug"),
            LogLevel::INFO => set_var("RUST_LOG", "info"),
            LogLevel::WARN => set_var("RUST_LOG", "warn"),
            LogLevel::ERROR => set_var("RUST_LOG", "error"),
        },
        None => set_var("RUST_LOG", "warn"),
    };
    pretty_env_logger::init_timed();

    match &opt.subcommand {
        SubCommands::Prep { input } => println!("{:?}", input),
    }
    Ok(())
}