#[macro_use]
extern crate log;

mod io;
mod args;
mod markdown;
mod posts;
mod server;
mod text_engine;

use anyhow::Result;
use markdown::dump::dump_doc;
use tantivy::Index;
use tantivy::collector::{Count, TopDocs};
use tantivy::query::AllQuery;
use std::env::set_var;
use std::fs;
use structopt::StructOpt;

use crate::args::{LogLevel, Opt, SubCommands};
use crate::markdown::template::template;
use crate::text_engine::query::{get_by_uuid, put};
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
        SubCommands::Prep { input, index_dir, rebuild } => {
            info!("input: {:?} index_dir: {:?}", input, index_dir);
            let glob_pattern = format!("{}/**/*.md", input.as_path().to_str().unwrap());
            info!("glob_pattern: {}", glob_pattern);
            let posts = posts::get_all_posts(&glob_pattern)?;
            println!("rebuld: {}", rebuild);
            let ja_schema = build_schema();
            let ja_index = read_or_build_index(ja_schema.clone(), index_dir, *rebuild)?;
            let mut ja_index_writer = ja_index.writer(100_000_000)?;

            info!("Find {} posts in {:?}", posts.len(), input);
            for post in posts.iter() {
                put(&post, &ja_index, &mut ja_index_writer)?;
            }

            let term = "fabe88b5-a35e-4954-bfd8-b5e88c585e7a";

            let doc = get_by_uuid(term, &ja_index)?;
            println!("term: {}\n{}", term, ja_schema.to_json(&doc));
        }

        SubCommands::Run {} => {}
        SubCommands::Template {} => {
            println!("{}", template());
        }

        SubCommands::Dump { outdir, index_dir} => {
            let index = Index::open_in_dir(index_dir)?;
            if !outdir.exists() {
                fs::create_dir(outdir)?;
            }
            let searcher = index.reader()?.searcher();
            let query = AllQuery {};
            let counter = Count {};

            let all_docs_number = searcher.search(&query, &counter)?;
            let docs = searcher.search(&query, &TopDocs::with_limit(all_docs_number))?;

            for (_, doc_address) in docs {
                let doc = searcher.doc(doc_address)?;
                let (filename, body) = dump_doc(&doc, &index.schema())?;

                let outfile = outdir.as_path().join(filename);
                let stem = outfile.parent().unwrap();
                if !stem.exists() {
                    fs::create_dir(stem)?;
                }
                io::write_string(&outfile, &body)?;
            }
        }
    }
    Ok(())
}
