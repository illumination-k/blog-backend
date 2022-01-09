use anyhow::{anyhow, Result};
use tantivy::{
    collector::TopDocs,
    query::{QueryParser, TermQuery},
    schema::Field,
    Document, Index, Term,
};

pub fn term_query(term: &str, field: Field, index: Index) -> Result<Document> {
    let reader = index.reader()?;
    let seracher = reader.searcher();

    let t = Term::from_field_text(field, term);
    let tq = TermQuery::new(t, tantivy::schema::IndexRecordOption::Basic);

    let docs = seracher.search(&tq, &TopDocs::with_limit(10))?;

    if docs.len() == 0 {
        return Err(anyhow!("{} is Not Found", term));
    }

    let (_, doc_address) = docs.into_iter().next().unwrap();
    let doc = seracher.doc(doc_address)?;
    Ok(doc)
}

pub fn get_by_uuid(uuid: &str, index: Index) -> Result<Document> {
    let field = index.schema().get_field("uuid").unwrap();
    term_query(uuid, field, index)
}

pub fn search(
    query: &str,
    fields: Vec<Field>,
    limit: usize,
    index: Index,
) -> Result<Vec<Document>> {
    let searcher = index.reader()?.searcher();
    let query_parser = QueryParser::for_index(&index, fields);
    let query = query_parser.parse_query(&query)?;

    let docs = searcher.search(&query, &TopDocs::with_limit(limit))?;

    Ok(docs
        .into_iter()
        .flat_map(|(_, doc_address)| searcher.doc(doc_address).ok())
        .collect())
}
