use actix_web::{get, web, HttpRequest, HttpResponse};
use itertools::Itertools;
use serde::Deserialize;
use std::ops::Deref;
use tantivy::Index;

use crate::text_engine::{
    query::search,
    schema::{FieldGetter, PostField},
};

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
    let fields = [PostField::Title, PostField::Description, PostField::RawText]
        .into_iter()
        .map(|pf| fb.get_field(pf))
        .collect_vec();

    let docs = search(&query.to_owned().to_lowercase(), fields, 10, index.deref()).unwrap();

    let docs = docs
        .iter()
        .map(|doc| index.schema().to_named_doc(doc))
        .collect_vec();
    HttpResponse::Ok().json(docs)
}
