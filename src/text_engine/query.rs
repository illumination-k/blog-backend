use anyhow::{anyhow, Result};
use chrono::Utc;
use itertools::Itertools;
use tantivy::{
    collector::{Count, TopDocs},
    query::{AllQuery, Query, QueryParser, TermQuery},
    schema::Field,
    Document, Index, IndexWriter, Term,
};

use crate::posts::Post;

use super::schema::{FieldGetter, PostField};

pub fn get_all(query: &dyn Query, index: &Index) -> Result<Option<Vec<Document>>> {
    let searcher = index.reader()?.searcher();
    let counter = Count {};
    let count = searcher.search(query, &counter)?;

    if count == 0 {
        return Ok(None);
    }
    let docs = searcher.search(query, &TopDocs::with_limit(count))?;

    Ok(Some(
        docs.into_iter()
            .flat_map(|(_, doc_address)| searcher.doc(doc_address).ok())
            .collect(),
    ))
}

pub fn get_tags_and_categories(index: &Index) -> Result<(Vec<String>, Vec<String>)> {
    let q: Box<dyn Query> = Box::new(AllQuery {});
    let schema = index.schema();
    let fq = FieldGetter::new(&schema);

    let _docs = get_all(&q, index)?;
    let tag_field = fq.get_field(PostField::Tags);
    let category_field = fq.get_field(PostField::Category);

    if let Some(docs) = _docs {
        let (categories_string, tags_string) =
            docs.iter()
                .fold((String::new(), String::new()), |(categories, tags), doc| {
                    let category = doc.get_first(category_field).unwrap().text().unwrap();
                    let inner_tags = doc.get_first(tag_field).unwrap().text().unwrap();

                    (
                        format!("{} {}", categories, category),
                        format!("{} {}", tags, inner_tags),
                    )
                });
        return Ok((
            categories_string
                .trim()
                .split(' ')
                .map(|x| x.to_string())
                .unique()
                .collect_vec(),
            tags_string
                .trim()
                .split(' ')
                .map(|x| x.to_string())
                .unique()
                .collect_vec(),
        ));
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
    let field = index.schema().get_field("uuid").unwrap();
    term_query_one(uuid, field, index)
}

pub fn put(post: &Post, index: &Index, index_writer: &mut IndexWriter) -> Result<Option<Document>> {
    let now = Utc::now();
    let schema = index.schema();
    let fb = FieldGetter::new(&schema);
    let new_doc = match get_by_uuid(&post.uuid(), index) {
        Ok(doc) => {
            let uuid_field = fb.get_field(PostField::Uuid);
            // dbg!(&post, &Post::from_doc(&doc, &index.schema()));
            // if no update in post, skip update index
            if post.equal_from_doc(&Post::from_doc(&doc, &schema)?) {
                info!("skip post: {}", post.title());
                return Ok(None);
            }

            let created_at = if let Some(created_at) = post.matter().created_at() {
                created_at
            } else {
                doc.get_first(fb.get_field(PostField::CreatedAt))
                    .unwrap()
                    .date_value()
                    .unwrap()
                    .to_owned()
            };

            let new_doc = post.to_doc(&index.schema(), &created_at, &now);
            index_writer.delete_term(Term::from_field_text(uuid_field, &post.uuid()));
            index_writer.add_document(new_doc.clone());
            new_doc
        }
        Err(_) => {
            // If no document in index, insert doc
            let created_at = if let Some(c) = post.created_at() {
                c
            } else {
                now
            };

            let updated_at = if let Some(u) = post.updated_at() {
                u
            } else {
                now
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
