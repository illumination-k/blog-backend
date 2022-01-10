use actix_web::{get, web, App, HttpRequest, HttpResponse, HttpServer, Responder};
use anyhow::Result;
use itertools::Itertools;
use serde::Deserialize;
use std::{ops::Deref, path::Path};
use tantivy::{
    query::{AllQuery, BooleanQuery, Occur, Query, TermQuery},
    schema::IndexRecordOption,
    Index, Term,
};

use crate::text_engine::{
    index::read_or_build_index,
    query::{get_all, search},
    schema::{build_schema, FieldGetter, PostField},
};

#[get("/")]
async fn hello() -> impl Responder {
    "Hello, Smark!"
}

#[derive(Debug, Deserialize)]
pub struct SearchQueryParams {
    query: String,
}

#[get("/posts/search")]
async fn search_posts(index: web::Data<Index>, req: HttpRequest) -> HttpResponse {
    let index = index.into_inner();
    let query = web::Query::<SearchQueryParams>::from_query(req.query_string())
        .unwrap()
        .query
        .to_owned();
    let fb = FieldGetter::new(index.schema());
    let fields = [PostField::Title, PostField::Description, PostField::RawText].into_iter().map(|pf| fb.get_field(pf)).collect_vec();

    let docs = search(
        &query.to_owned().to_lowercase(),
        fields,
        10,
        index.deref(),
    )
    .unwrap();

    let docs = docs
        .iter()
        .map(|doc| index.schema().to_named_doc(doc))
        .collect_vec();
    HttpResponse::Ok().json(docs)
}

#[derive(Debug, Deserialize)]
pub struct GetPostsQueryParams {
    lang: Option<String>,
    category: Option<String>,
    tag: Option<String>,
}

#[get("/posts")]
async fn get_posts(index: web::Data<Index>, req: HttpRequest) -> HttpResponse {
    let index = index.into_inner();
    let schema = index.schema();
    let params = web::Query::<GetPostsQueryParams>::from_query(req.query_string()).unwrap();

    let mut queries = vec![];
    if let Some(lang) = params.lang.to_owned() {
        let lang_field = schema.get_field("lang").unwrap();
        let term = Term::from_field_text(lang_field, &lang);
        let query: Box<dyn Query> = Box::new(TermQuery::new(term, IndexRecordOption::Basic));
        queries.push((Occur::Must, query));
    }

    if let Some(category) = params.category.to_owned() {
        let category_field = schema.get_field("category").unwrap();
        let term = Term::from_field_text(category_field, &category);
        let query: Box<dyn Query> = Box::new(TermQuery::new(term, IndexRecordOption::Basic));

        queries.push((Occur::Must, query));
    }

    if let Some(tag) = params.tag.to_owned() {
        let tag_field = schema.get_field("tag").unwrap();
        let term = Term::from_field_text(tag_field, &tag);
        let query: Box<dyn Query> = Box::new(TermQuery::new(term, IndexRecordOption::Basic));

        queries.push((Occur::Must, query));
    }

    let docs = if queries.is_empty() {
        let q: Box<dyn Query> = Box::new(AllQuery {});
        get_all(&q, index.deref())
    } else {
        let q: Box<dyn Query> = Box::new(BooleanQuery::new(queries));
        get_all(&q, index.deref())
    }
    .unwrap()
    .iter()
    .map(|doc| index.schema().to_named_doc(doc))
    .collect_vec();

    HttpResponse::Ok().json(docs)
}

#[actix_web::main]
pub async fn main(port: String) -> Result<()> {
    let schema = build_schema();
    let index = read_or_build_index(schema, &Path::new("test/index"), false)?;
    HttpServer::new(move || {
        App::new()
            .data(index.clone())
            .service(get_posts)
            .service(search_posts)
            .service(hello)
    })
    .bind(&format!("127.0.0.1:{}", port))?
    .run()
    .await
    .expect("Error in build httpserver");
    Ok(())
}
