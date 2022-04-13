use std::collections::HashSet;

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};

use tantivy::{
    collector::{Count, TopDocs},
    query::{AllQuery, BooleanQuery, Occur, Query, QueryParser, TermQuery},
    schema::Field,
    DocAddress, Document, Index, IndexWriter, Term,
};

use crate::posts::Post;

use super::schema::{FieldGetter, PostField};
use crate::datetime::{self, DateTimeFormat, DateTimeWithFormat};

pub fn get_all(
    query: &dyn Query,
    index: &Index,
    order_by: Option<datetime::OrderBy>,
) -> Result<Option<Vec<Document>>> {
    let schema = index.schema();
    let searcher = index.reader()?.searcher();
    let counter = Count {};
    let count = searcher.search(query, &counter)?;

    let fb = FieldGetter::new(&schema);
    if count == 0 {
        return Ok(None);
    }

    let docs = if let Some(order_by) = order_by {
        let collector =
            match order_by {
                datetime::OrderBy::CreatedAt => TopDocs::with_limit(count)
                    .order_by_fast_field(fb.get_field(PostField::CreatedAt)),
                datetime::OrderBy::UpdatedAt => TopDocs::with_limit(count)
                    .order_by_fast_field(fb.get_field(PostField::UpdatedAt)),
            };
        searcher
            .search(query, &collector)?
            .into_iter()
            .flat_map(|doc: (DateTime<Utc>, DocAddress)| searcher.doc(doc.1).ok())
            .collect()
    } else {
        searcher
            .search(query, &TopDocs::with_limit(count))?
            .into_iter()
            .flat_map(|(_, doc_address)| searcher.doc(doc_address).ok())
            .collect()
    };

    Ok(Some(docs))
}

pub fn get_tags_and_categories(index: &Index) -> Result<(Vec<String>, Vec<String>)> {
    let q: Box<dyn Query> = Box::new(AllQuery {});
    let schema = index.schema();
    let fg = FieldGetter::new(&schema);

    let _docs = get_all(&q, index, None)?;

    if let Some(docs) = _docs {
        let mut categories = HashSet::new();
        let mut tags = HashSet::new();

        for doc in docs.iter() {
            let category = fg.get_text(doc, PostField::Category)?;
            let inner_tags = fg.get_tags(doc)?;

            categories.insert(category);
            tags.extend(inner_tags.into_iter())
        }

        return Ok((tags.into_iter().collect(), categories.into_iter().collect()));
    }

    Ok((Vec::new(), Vec::new()))
}

pub fn term_query_one(term: &str, field: Field, index: &Index) -> Result<Document> {
    let reader = index.reader()?;
    let seracher = reader.searcher();

    let t = Term::from_field_text(field, term);
    let tq = TermQuery::new(t, tantivy::schema::IndexRecordOption::Basic);

    let docs = seracher.search(&tq, &TopDocs::with_limit(10))?;

    if docs.is_empty() {
        return Err(anyhow!("{} is Not Found", term));
    }

    let (_, doc_address) = docs.into_iter().next().unwrap();
    let doc = seracher.doc(doc_address)?;
    Ok(doc)
}

pub fn get_by_uuid(uuid: &str, index: &Index) -> Result<Document> {
    let schema = index.schema();
    let fg = FieldGetter::new(&schema);
    let field = fg.get_field(PostField::Uuid);
    term_query_one(uuid, field, index)
}

pub fn get_by_slug_with_lang(slug: &str, lang: &str, index: &Index) -> Result<Document> {
    let reader = index.reader()?;
    let searcher = reader.searcher();
    let schema = index.schema();
    let fg = FieldGetter::new(&schema);
    let slug_field = fg.get_field(PostField::Slug);
    let lang_field = fg.get_field(PostField::Lang);

    let slug_query: Box<dyn Query> = Box::new(TermQuery::new(
        Term::from_field_text(slug_field, slug),
        tantivy::schema::IndexRecordOption::Basic,
    ));
    let lang_query: Box<dyn Query> = Box::new(TermQuery::new(
        Term::from_field_text(lang_field, lang),
        tantivy::schema::IndexRecordOption::Basic,
    ));

    let q = BooleanQuery::new(vec![(Occur::Must, slug_query), (Occur::Must, lang_query)]);

    let docs = searcher.search(&q, &TopDocs::with_limit(1))?;
    if docs.is_empty() {
        return Err(anyhow!("slug: {} and lang: {} is Not Found", slug, lang));
    }

    let (_, doc_address) = docs.into_iter().next().unwrap();
    Ok(searcher.doc(doc_address)?)
}

pub fn put(
    post: &Post,
    index: &Index,
    index_writer: &mut IndexWriter,
    skip_update_date: bool,
) -> Result<Option<Document>> {
    let now = Utc::now();

    let schema = index.schema();
    let fb = FieldGetter::new(&schema);
    let new_doc = match get_by_uuid(&post.uuid(), index) {
        Ok(doc) => {
            let uuid_field = fb.get_field(PostField::Uuid);
            // if no update in post, skip update index
            // post.diff(&Post::from_doc(&doc, &schema)?);
            if post.equal_from_doc(&Post::from_doc(&doc, &schema)?) {
                info!("skip post: {}", post.title());
                return Ok(None);
            }

            let created_at = if let Some(created_at) = post.matter().created_at() {
                created_at
            } else {
                let datetime = fb.get_date(&doc, PostField::CreatedAt)?;
                let format = fb.get_text(&doc, PostField::CreatedAtFormat)?;
                DateTimeWithFormat::new(datetime, DateTimeFormat::from(format.as_str()))
            };

            let updated_at = if skip_update_date {
                post.updated_at().unwrap()
            } else {
                let updated_at_format =
                    DateTimeFormat::from(fb.get_text(&doc, PostField::UpdatedAtFormat)?.as_str());
                DateTimeWithFormat::new(now, updated_at_format)
            };

            let new_doc = post.to_doc(&schema, &created_at, &updated_at);
            index_writer.delete_term(Term::from_field_text(uuid_field, &post.uuid()));
            index_writer.add_document(new_doc.clone());
            new_doc
        }
        Err(_) => {
            // If no document in index, insert doc
            let now_with_format = DateTimeWithFormat::new(now, DateTimeFormat::RFC3339);
            let created_at = if let Some(c) = post.created_at() {
                c
            } else {
                now_with_format.clone()
            };

            let updated_at = if let Some(u) = post.updated_at() {
                u
            } else {
                now_with_format
            };

            let new_doc = post.to_doc(&index.schema(), &created_at, &updated_at);
            index_writer.add_document(new_doc.clone());
            new_doc
        }
    };
    index_writer.commit()?;
    Ok(Some(new_doc))
}

pub fn search(
    query: &str,
    fields: Vec<Field>,
    limit: usize,
    index: &Index,
) -> Result<Vec<Document>> {
    if limit == 0 {
        return Ok(Vec::new());
    }
    let searcher = index.reader()?.searcher();
    let query_parser = QueryParser::for_index(index, fields);
    let query = query_parser.parse_query(query)?;

    let docs = searcher.search(&query, &TopDocs::with_limit(limit))?;

    Ok(docs
        .into_iter()
        .flat_map(|(_, doc_address)| searcher.doc(doc_address).ok())
        .collect())
}

#[cfg(test)]
mod test {
    use crate::test_utility::*;

    use super::*;
    use crate::posts::frontmatter::FrontMatter;
    use crate::test_utility::build_random_posts_index;
    use crate::{
        posts::Post,
        text_engine::{index::read_or_build_index, schema::build_schema},
    };
    use tempdir::TempDir;

    #[test]
    fn test_get_all() -> Result<()> {
        let temp_dir = TempDir::new(&format!(
            "temp_rand_index_{}",
            uuid::Uuid::new_v4().to_string()
        ))?;
        let (posts, index) = build_random_posts_index(10, temp_dir.path())?;
        assert_eq!(posts.len(), 10);

        let q: Box<dyn Query> = Box::new(AllQuery {});
        let docs = get_all(&q, &index, None)?;
        assert!(docs.is_some());
        assert_eq!(docs.unwrap().len(), 10);
        Ok(())
    }

    #[test]
    fn test_get_by_uuid() -> Result<()> {
        let temp_dir = TempDir::new(&format!(
            "temp_rand_index_{}",
            uuid::Uuid::new_v4().to_string()
        ))?;
        let schema = build_schema();
        let (posts, index) = build_random_posts_index(5, temp_dir.path())?;

        for post in posts.iter() {
            let uuid = post.uuid();
            let doc = get_by_uuid(&uuid, &index)?;
            let res_post = Post::from_doc(&doc, &schema)?;
            assert_eq!(res_post.uuid(), uuid);
            assert!(res_post.equal_from_doc(&post));
        }

        Ok(())
    }

    #[test]
    fn test_get_by_slug_and_lang() -> Result<()> {
        let temp_dir = TempDir::new(&format!(
            "temp_rand_index_{}",
            uuid::Uuid::new_v4().to_string()
        ))?;
        let schema = build_schema();
        let (posts, index) = build_random_posts_index(5, temp_dir.path())?;

        for post in posts.iter() {
            let slug = post.slug();
            let lang = post.lang();

            let doc = get_by_slug_with_lang(&slug, lang.as_str(), &&index)?;
            let res_post = Post::from_doc(&doc, &schema)?;

            assert!(post.equal_from_doc(&res_post));
        }

        Ok(())
    }

    #[test]
    fn test_get_tags_and_categories() -> Result<()> {
        let temp_dir = tempdir::TempDir::new(&format!(
            "temp_rand_index_{}",
            uuid::Uuid::new_v4().to_string()
        ))?;
        let (posts, index) = build_random_posts_index(10, temp_dir.path())?;

        let (mut tags, mut categories) = get_tags_and_categories(&index)?;

        let mut rand_tags = HashSet::new();
        let mut rand_categories = HashSet::new();

        for post in posts.iter() {
            rand_categories.insert(post.category());
            if let Some(tags) = post.tags() {
                for tag in tags.iter() {
                    rand_tags.insert(tag.to_owned());
                }
            }
        }

        let mut rand_tags: Vec<String> = rand_tags.into_iter().collect();
        let mut rand_categories: Vec<String> = rand_categories.into_iter().collect();
        tags.sort_unstable();
        categories.sort_unstable();
        rand_tags.sort_unstable();
        rand_categories.sort_unstable();

        assert_eq!(tags, rand_tags);
        assert_eq!(categories, rand_categories);

        Ok(())
    }

    #[test]
    fn test_put() {
        let temp_dir = TempDir::new("test_put_query").unwrap();

        let schema = build_schema();
        let index =
            read_or_build_index(schema.clone(), &temp_dir.path().join("put"), false).unwrap();
        let mut index_writer = index.writer(100_000_000).unwrap();
        let post = rand_post();
        let doc = put(&post, &index, &mut index_writer, false)
            .unwrap()
            .unwrap();
        let post_doc = Post::from_doc(&doc, &schema).unwrap();
        assert!(post.equal_from_doc(&post_doc));

        let none = put(&post_doc, &index, &mut index_writer, false).unwrap();
        assert!(none.is_none());
    }

    #[test]
    fn test_put_with_format() -> Result<()> {
        use crate::test_utility::*;
        let temp_dir = tempdir::TempDir::new(&format!(
            "temp_rand_index_{}",
            uuid::Uuid::new_v4().to_string()
        ))?;

        fn rand_post_with_format() -> Post {
            let now = Utc::now();
            let tags = rand_tags(3);
            let matter = FrontMatter::new(
                uuid::Uuid::new_v4(),
                rand_japanase(TITLE_LENGTH),
                rand_japanase(DESCRIPTION_LENGTH),
                rand_japanase(TAG_CATEGORIES_LENGTH),
                rand_lang(),
                tags,
                Some(DateTimeWithFormat::new(
                    now,
                    DateTimeFormat::Custom("YY/MM/DD".to_string()),
                )),
                Some(DateTimeWithFormat::new(
                    now,
                    DateTimeFormat::Custom("YY-MM-DD".to_string()),
                )),
            );
            Post::new(rand_alpahbet(10), matter, rand_japanase(BODY_LENGHT))
        }

        let schema = build_schema();
        let index = read_or_build_index(schema, temp_dir.path(), true)?;
        let mut index_writer = index.writer(100_000_000)?;

        fn test_put(post: &mut Post, index: &Index, index_writer: &mut IndexWriter) -> Result<()> {
            let doc = put(&post, &index, index_writer, false)?;
            assert!(doc.is_some());
            let prev_post = Post::from_doc(&doc.unwrap(), &index.schema())?;

            let body = post.body_mut();
            *body = rand_japanase(BODY_LENGHT - 10);

            let doc = put(&post, &index, index_writer, false)?;
            assert!(doc.is_some());
            let updated_post = Post::from_doc(&doc.unwrap(), &index.schema())?;

            assert_ne!(prev_post.updated_at(), updated_post.updated_at());
            assert_eq!(
                prev_post.updated_at().unwrap().format(),
                updated_post.updated_at().unwrap().format()
            );
            Ok(())
        }

        let mut post_with_format = rand_post_with_format();
        test_put(&mut post_with_format, &index, &mut index_writer)?;

        let mut post = rand_post();
        test_put(&mut post, &index, &mut index_writer)?;

        Ok(())
    }
}
