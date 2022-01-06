use std::path::PathBuf;

use structopt::{clap, clap::arg_enum, StructOpt};

#[derive(Debug, StructOpt)]
#[structopt(name = "corrnet")]
#[structopt(long_version(option_env!("LONG_VERSION").unwrap_or(env!("CARGO_PKG_VERSION"))))]
#[structopt(setting(clap::AppSettings::ColoredHelp))]
pub struct Opt {
    #[structopt(long = "log", possible_values(&LogLevel::variants()))]
    pub log_level: Option<LogLevel>,
    #[structopt(subcommand)]
    pub subcommand: SubCommands,
}

arg_enum! {
    #[derive(Debug)]
    pub enum LogLevel {
        DEBUG,
        INFO,
        WARN,
        ERROR,
    }
}

#[derive(Debug, StructOpt)]
pub enum SubCommands {
    #[structopt(
        name = "prep",
        about = "Preperation to run server from markdown"
    )]
    #[structopt(setting(clap::AppSettings::ColoredHelp))]
    Prep {
        #[structopt(short = "-i", long = "input")]
        input: PathBuf,
    }
}