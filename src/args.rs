use std::path::PathBuf;
use structopt::{clap, clap::arg_enum, StructOpt};

#[derive(Debug, StructOpt)]
#[structopt(name = "smark")]
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
        about = "Preperation to run server from markdown by indexing"
    )]
    #[structopt(setting(clap::AppSettings::ColoredHelp))]
    Prep {
        #[structopt(short = "-i", long = "input")]
        input: PathBuf,
        #[structopt(long = "index_dir")]
        index_dir: PathBuf,
        #[structopt(long = "rebuild")]
        rebuild: bool,
    },

    #[structopt(name = "run", about = "run server")]
    #[structopt(setting(clap::AppSettings::ColoredHelp))]
    Run {},

    #[structopt(name = "template", about = "stdout template")]
    #[structopt(setting(clap::AppSettings::ColoredHelp))]
    Template {},

    #[structopt(
        name = "dump",
        about = "dump markdown from index with created_at and updated_at"
    )]
    #[structopt(setting(clap::AppSettings::ColoredHelp))]
    Dump {
        #[structopt(short = "-o", long = "outdir")]
        outdir: PathBuf,
        #[structopt(long = "index_dir")]
        index_dir: PathBuf,
    },
}
