#[macro_use]
extern crate log;

mod args;
mod datetime;
mod io;
mod posts;
mod server;
mod text_engine;

#[cfg(test)]
mod test_utility;

use anyhow::{anyhow, Result};
use std::env::set_var;
use std::fs;
use structopt::StructOpt;
use tantivy::collector::{Count, TopDocs};
use tantivy::query::AllQuery;
use tantivy::Index;

use datetime::DateTimeWithFormat;
use posts::dump::{dump_doc, dump_post};
use posts::frontmatter::{parse_date_with_format, replace_frontmatter};

use crate::args::{LogLevel, Opt, SubCommands};
use crate::io::{read_string, write_string};
use crate::posts::frontmatter::find_frontmatter_block;
use crate::posts::template;
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
        SubCommands::Prep {
            input,
            index_dir,
            rebuild,
        } => {
            let glob_pattern = format!("{}/**/*.md", input.display());
            eprintln!("---- Prep Parmeters  ----");
            eprintln!(
                "input: {:?}, index_dir: {:?}, glob_pattern: {}, rebuild: {}",
                input, index_dir, glob_pattern, rebuild
            );
            let schema = build_schema();
            let index = read_or_build_index(schema, index_dir, *rebuild)?;

            posts::index::build(&glob_pattern, &index)?;
        }

        SubCommands::Run {
            port,
            host,
            index_dir,
            _cors_origin,
            static_dir,
        } => {
            if !static_dir.exists() {
                return Err(anyhow!(format!("{} does not exist", static_dir.display())));
            }
            server::main(
                host.to_owned(),
                port.to_string(),
                index_dir.to_owned(),
                static_dir.to_owned(),
                _cors_origin.to_owned(),
            )?;
        }
        SubCommands::Template {
            with_date,
            datetime_format,
        } => {
            print!("{}", template(with_date, datetime_format)?);
        }
        SubCommands::Replace {
            input,
            uuid,
            title,
            description,
            category,
            lang,
            tags,
            created_at,
            updated_at,
            write,
        } => {
            eprintln!("input: {:?}", input);
            let text = read_string(input)?;
            let (matter, body) = match find_frontmatter_block(&text) {
                Some((fm_start, fm_end)) => (&text[fm_start..fm_end], &text[fm_end..]),
                None => ("", text.as_str()),
            };

            let created_at = Some(if let Some(created_at) = created_at {
                parse_date_with_format(created_at)
            } else {
                DateTimeWithFormat::default()
            });

            let updated_at = Some(if let Some(updated_at) = updated_at {
                parse_date_with_format(updated_at)
            } else {
                DateTimeWithFormat::default()
            });

            let matter = replace_frontmatter(
                matter,
                uuid,
                title,
                category,
                lang,
                description,
                tags,
                &created_at,
                &updated_at,
            )?;

            let post = posts::Post::new(posts::path_to_slug(input), matter, body.to_string());

            let (_, content) = dump_post(&post)?;
            if *write {
                write_string(input, &content)?;
            } else {
                println!("{content}")
            };
        }
        SubCommands::Dump { outdir, index_dir } => {
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
