use actix_web::{get, web, HttpRequest, HttpResponse};
use itertools::Itertools;
use serde::Deserialize;
use std::ops::Deref;
use tantivy::{
    query::{AllQuery, BooleanQuery, Occur, Query, TermQuery},
    schema::IndexRecordOption,
    Index, Term,
};

use crate::text_engine::{
    query::{get_all, get_by_uuid},
    schema::{FieldGetter, PostField},
};

#[get("/posts/{uuid}")]
async fn get_post_by_id(index: web::Data<Index>, uuid: web::Path<String>) -> HttpResponse {
    let schema = index.schema();
    match get_by_uuid(&uuid.to_owned(), index.into_inner().deref()) {
        Ok(doc) => HttpResponse::Ok().json(schema.to_named_doc(&doc)),
        Err(e) => HttpResponse::NotFound().body(e.to_string()),
    }
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
    let fb = FieldGetter::new(&schema);
    let params = match web::Query::<GetPostsQueryParams>::from_query(req.query_string()) {
        Ok(p) => p,
        Err(e) => return HttpResponse::BadRequest().body(e.to_string()),
    };

    let mut queries = vec![];
    if let Some(lang) = params.lang.to_owned() {
        let lang_field = fb.get_field(PostField::Lang);
        let term = Term::from_field_text(lang_field, &lang);
        let query: Box<dyn Query> = Box::new(TermQuery::new(term, IndexRecordOption::Basic));
        queries.push((Occur::Must, query));
    }

    if let Some(category) = params.category.to_owned() {
        let category_field = fb.get_field(PostField::Category);
        let term = Term::from_field_text(category_field, &category);
        let query: Box<dyn Query> = Box::new(TermQuery::new(term, IndexRecordOption::Basic));

        queries.push((Occur::Must, query));
    }

    if let Some(tag) = params.tag.to_owned() {
        let tag_field = fb.get_field(PostField::Tags);
        let term = Term::from_field_text(tag_field, &tag);
        let query: Box<dyn Query> = Box::new(TermQuery::new(term, IndexRecordOption::Basic));

        queries.push((Occur::Must, query));
    }

    let __docs = if queries.is_empty() {
        let q: Box<dyn Query> = Box::new(AllQuery {});
        get_all(&q, index.deref())
    } else {
        let q: Box<dyn Query> = Box::new(BooleanQuery::new(queries));
        get_all(&q, index.deref())
    };

    let _docs = match __docs {
        Ok(_docs) => _docs,
        Err(_) => return HttpResponse::InternalServerError().body("Internal Server Error"),
    };

    let docs = if let Some(docs) = _docs {
        docs.iter()
            .map(|doc| index.schema().to_named_doc(doc))
            .collect_vec()
    } else {
        Vec::new()
    };

    HttpResponse::Ok().json(docs)
}
