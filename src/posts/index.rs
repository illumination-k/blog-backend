use anyhow::Result;
use tantivy::{Index, IndexWriter};

use super::utils::get_all_posts;
use super::Post;
use crate::io;
use crate::posts::dump::dump_post;
use crate::text_engine::query::put;
use crate::text_engine::schema::FieldGetter;

fn prep_post_index(
    post: &mut Post,
    fg: &FieldGetter,
    index: &Index,
    index_writer: &mut IndexWriter,
    skip_update_date: bool,
) -> Result<Option<String>> {
    let doc = put(post, index, index_writer, skip_update_date)?;
    if let Some(doc) = doc {
        let updated_at =
            fg.get_date_with_format(&doc, crate::text_engine::schema::PostField::UpdatedAt)?;
        *post.updated_at_mut() = Some(updated_at);

        let (_, new_markdown) = dump_post(post)?;
        Ok(Some(new_markdown))
    } else {
        Ok(None)
    }
}

pub fn build(glob_pattern: &str, index: &Index, skip_update_date: bool) -> Result<()> {
    let schema = index.schema();
    let fg = FieldGetter::new(&schema);
    let mut index_writer = index.writer(100_000_000)?;
    let mut posts = get_all_posts(glob_pattern)?;

    eprintln!("\n--- Start Preperation ---");
    eprintln!("- Find {} posts", posts.len());
    let mut update_post_count = 0;

    for (path, post) in posts.iter_mut() {
        if let Some(new_markdown) =
            prep_post_index(post, &fg, index, &mut index_writer, skip_update_date)?
        {
            update_post_count += 1;
            io::write_string(path, &new_markdown)?;
        }
    }

    eprintln!("- Update {} posts in this prepartion", update_post_count);
    eprintln!("-------- Finish! --------");
    Ok(())
}

#[cfg(test)]
mod test {
    use crate::{
        test_utility::*,
        text_engine::{query::search, schema::PostField},
    };
    use glob::glob;
    use tantivy::query::{AllQuery, Query};
    use tempdir::TempDir;

    use super::*;
    use crate::text_engine::{index::read_or_build_index, query::get_all, schema::build_schema};

    #[test]
    fn test_prep_post_index() -> Result<()> {
        let temp_dir = TempDir::new(&format!(
            "temp_rand_index_{}",
            uuid::Uuid::new_v4().to_string()
        ))?;
        let (mut posts, index) = build_random_posts_index(5, temp_dir.path())?;
        let mut index_writer = index.writer(100_000_000)?;
        let schema = index.schema();
        let fg = FieldGetter::new(&schema);

        let target_body = posts[0].body_mut();
        *target_body = "<!-- comment --> abc".to_string();
        let (_, old_markdown) = dump_post(&posts[0])?;

        // need skip update date because subtle change occurs
        let new_markdown =
            prep_post_index(&mut posts[0], &fg, &index, &mut index_writer, true)?.unwrap();

        assert_eq!(old_markdown, new_markdown);

        assert!(search("comment", vec![fg.get_field(PostField::Body)], 10, &index)?.is_empty());
        assert!(!search("abc", vec![fg.get_field(PostField::Body)], 10, &index)?.is_empty());

        Ok(())
    }

    #[test]
    fn test_build() -> Result<()> {
        let index_dir = TempDir::new("test_index_build")?;
        let glob_pattern = "test/posts/**/*.md";

        let schema = build_schema();
        let index = read_or_build_index(schema, index_dir.path(), true)?;

        build(glob_pattern, &index, false)?;
        let q: Box<dyn Query> = Box::new(AllQuery {});
        let docs = get_all(&q, &index, None)?;

        assert!(docs.is_some());
        let actual_files_count = glob(glob_pattern)?.filter(|x| x.is_ok()).count();
        assert_eq!(docs.unwrap().len(), actual_files_count);
        Ok(())
    }
}
