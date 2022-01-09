mod args;
mod markdown;
mod schema;
mod utils;

use anyhow::Result;
use std::env::set_var;
use structopt::StructOpt;

use crate::args::{LogLevel, Opt, SubCommands};
use crate::markdown::template::template;
use crate::schema::{build::build_schema, index::read_or_build_index};

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
            println!("input: {:?}\nindex_dir: {:?}", input, index_dir);
            let glob_pattern = format!("{}/**/*.md", input.as_path().to_str().unwrap());
            println!("glob_pattern: {}", glob_pattern);
            let posts = utils::get_all_posts(&glob_pattern)?;
            dbg!(&posts);

            let ja_schema = build_schema();
            let ja_index = read_or_build_index(ja_schema.clone(), index_dir, true)?;
            let mut ja_index_writer = ja_index.writer(100_000_000)?;

            for post in posts.iter() {
                if post.lang() == "ja" {
                    ja_index_writer.add_document(post.to_doc(&ja_schema));
                    ja_index_writer.commit()?;
                }
            }
        }

        SubCommands::Run {} => {}
        SubCommands::Template {} => {
            println!("{}", template());
        }
    }
    Ok(())
}
