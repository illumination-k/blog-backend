use anyhow::Result;
use tantivy::Index;

use super::utils::get_all_posts;
use crate::io;
use crate::posts::dump::dump_doc;
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

#[cfg(test)]
mod test {
    use tantivy::query::{Query, AllQuery};
    use tempdir::TempDir;
    use glob::glob;

    use super::*;
    use crate::text_engine::{index::read_or_build_index, schema::build_schema, query::get_all};

    #[test]
    fn test_build() -> Result<()> {
        let index_dir = TempDir::new("test_index_build")?;
        let glob_pattern = "test/posts/**/*.md";

        let schema = build_schema();
        let index = read_or_build_index(schema, index_dir.path(), true)?;
        
        build(glob_pattern, &index)?;
        let q: Box<dyn Query> = Box::new(AllQuery {});
        let docs = get_all(&q, &index, None)?;

        assert!(docs.is_some());
        let actual_files_count = glob(glob_pattern)?.filter(|x| x.is_ok()).count();
        assert_eq!(docs.unwrap().len(), actual_files_count);
        Ok(())
    }
}