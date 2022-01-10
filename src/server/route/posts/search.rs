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
    query: Option<String>,
    limit: Option<usize>,
}

#[get("/posts/search")]
async fn search_posts(index: web::Data<Index>, req: HttpRequest) -> HttpResponse {
    let index = index.into_inner();
    let params = web::Query::<SearchQueryParams>::from_query(req.query_string())
        .unwrap();
    
    let fb = FieldGetter::new(index.schema());
    let fields = [PostField::Title, PostField::Description, PostField::RawText]
        .into_iter()
        .map(|pf| fb.get_field(pf))
        .collect_vec();
    
    let limit = if let Some(limit) = params.limit.to_owned() {
        limit
    } else {
        10
    };

    let docs = if let Some(query) = params.query.to_owned() {
        search(&query.to_owned().to_lowercase(), fields, limit, index.deref())
            .unwrap()
            .iter()
            .map(|doc| index.schema().to_named_doc(doc))
            .collect_vec()
    } else {
        Vec::new()
    };
    HttpResponse::Ok().json(docs)
}
