use anyhow::Result;
use tantivy::Index;

use super::reader::get_all_posts;
use crate::io;
use crate::markdown::dump::dump_doc;
use crate::text_engine::query::put;

pub fn build(glob_pattern: &str, index: &Index) -> Result<()> {
    let schema = index.schema();
    let mut index_writer = index.writer(100_000_000)?;
    let posts = get_all_posts(glob_pattern)?;

    eprintln!("\n--- Start Preperation ---");
    eprintln!("- Find {} posts", posts.len());
    let mut update_post_count = 0;

    for (path, post) in posts.iter() {
        let doc = put(post, index, &mut index_writer)?;
        if let Some(doc) = doc {
            update_post_count += 1;

            let (_, new_markdown) = dump_doc(&doc, &schema)?;
            io::write_string(path, &new_markdown)?;
        }
    }

    eprintln!("- Update {} posts in this prepartion", update_post_count);
    eprintln!("-------- Finish! --------");
    Ok(())
}
