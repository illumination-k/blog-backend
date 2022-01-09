use anyhow::{anyhow, Result};
use chrono::Utc;
use tantivy::{
    collector::TopDocs,
    query::{QueryParser, TermQuery},
    schema::Field,
    Document, Index, IndexWriter, Term,
};

use crate::posts::Post;

pub fn term_query(term: &str, field: Field, index: &Index) -> Result<Document> {
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
    term_query(uuid, field, index)
}

pub fn put(post: &Post, index: &Index, index_writer: &mut IndexWriter) -> Result<()> {
    let now = Utc::now();
    debug!("{}", now);
    match get_by_uuid(&post.uuid(), index) {
        Ok(doc) => {
            let uuid_field = index.schema().get_field("uuid").unwrap();
            
            // if no update in post, skip update index
            if post == &Post::from_doc(&doc, &index.schema()) {
                info!("skip post: {}", post.title());
                return Ok(());
            }

            let created_at = doc
                .get_first(index.schema().get_field("created_at").unwrap())
                .unwrap()
                .date_value()
                .unwrap();
            index_writer.delete_term(Term::from_field_text(uuid_field, &post.uuid()));
            index_writer.add_document(post.to_doc(&index.schema(), created_at, &now));
        }
        Err(_) => {
            // If no document in index, insert doc
            index_writer.add_document(post.to_doc(&index.schema(), &now, &now));
        }
    }
    index_writer.commit()?;
    Ok(())
}

pub fn search(
    query: &str,
    fields: Vec<Field>,
    limit: usize,
    index: Index,
) -> Result<Vec<Document>> {
    let searcher = index.reader()?.searcher();
    let query_parser = QueryParser::for_index(&index, fields);
    let query = query_parser.parse_query(query)?;

    let docs = searcher.search(&query, &TopDocs::with_limit(limit))?;

    Ok(docs
        .into_iter()
        .flat_map(|(_, doc_address)| searcher.doc(doc_address).ok())
        .collect())
}
