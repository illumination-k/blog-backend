#[macro_use]
extern crate log;

mod args;
mod markdown;
mod server;
mod text_engine;
mod utils;

use anyhow::Result;
use std::env::set_var;
use structopt::StructOpt;

use crate::args::{LogLevel, Opt, SubCommands};
use crate::markdown::template::template;
use crate::text_engine::query::{term_query, get_by_uuid};
use crate::text_engine::{index::read_or_build_index, schema::build_schema};

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
        SubCommands::Prep { input, index_dir } => {
            info!("input: {:?} index_dir: {:?}", input, index_dir);
            let glob_pattern = format!("{}/**/*.md", input.as_path().to_str().unwrap());
            info!("glob_pattern: {}", glob_pattern);
            let posts = utils::get_all_posts(&glob_pattern)?;

            let ja_schema = build_schema();
            let ja_index = read_or_build_index(ja_schema.clone(), index_dir, true)?;
            let mut ja_index_writer = ja_index.writer(100_000_000)?;

            info!("Find {} posts in {:?}", posts.len(), input);
            for post in posts.iter() {
                ja_index_writer.add_document(post.to_doc(&ja_schema));
                ja_index_writer.commit()?;
            }

            let term = "fabe88b5-a35e-4954-bfd8-b5e88c585e7a";
            let target = ja_schema.get_field("uuid").unwrap();

            let doc = get_by_uuid(term, ja_index)?;
            println!("term: {}\n{}", term, ja_schema.to_json(&doc));
        }

        SubCommands::Run {} => {}
        SubCommands::Template {} => {
            println!("{}", template());
        }
    }
    Ok(())
}
