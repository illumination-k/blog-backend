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

use crate::text_engine::query::OrderBy;

#[get("/posts/{uuid}")]
async fn get_post_by_id(index: web::Data<Index>, uuid: web::Path<String>) -> HttpResponse {
    let schema = index.schema();
    match get_by_uuid(&uuid.to_owned(), index.into_inner().deref()) {
        Ok(doc) => HttpResponse::Ok().json(schema.to_named_doc(&doc)),
        Err(e) => HttpResponse::NotFound().body(e.to_string()),
    }
}

#[derive(Debug, Clone, Deserialize)]
pub enum Order {
    Asc,
    Desc,
}

#[derive(Debug, Deserialize)]
pub struct GetPostsQueryParams {
    lang: Option<String>,
    category: Option<String>,
    tag: Option<String>,
    order_by: Option<OrderBy>,
    order: Option<Order>,
}

impl GetPostsQueryParams {
    pub fn to_queries(&self, fb: &FieldGetter) -> Vec<(Occur, Box<dyn Query>)> {
        [
            self.lang.to_owned(),
            self.category.to_owned(),
            self.tag.to_owned(),
        ]
        .into_iter()
        .flatten()
        .zip(&[PostField::Lang, PostField::Category, PostField::Tags])
        .map(|(val, &pf)| {
            let field = fb.get_field(pf);
            let q: Box<dyn Query> = Box::new(TermQuery::new(
                Term::from_field_text(field, &val),
                IndexRecordOption::Basic,
            ));
            (Occur::Must, q)
        })
        .collect()
    }

    pub fn order_by(&self) -> Option<OrderBy> {
        self.order_by.to_owned()
    }

    pub fn get_order(&self) -> Order {
        self.order.to_owned().unwrap_or(Order::Desc)
    }
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

    let queries = params.to_queries(&fb);

    let __docs = if queries.is_empty() {
        let q: Box<dyn Query> = Box::new(AllQuery {});
        get_all(&q, index.deref(), params.order_by())
    } else {
        let q: Box<dyn Query> = Box::new(BooleanQuery::new(queries));
        get_all(&q, index.deref(), params.order_by())
    };

    let _docs = match __docs {
        Ok(_docs) => _docs,
        Err(e) => {
            error!("{:?}", e);
            return HttpResponse::InternalServerError().body("Internal Server Error");
        }
    };

    let mut docs = if let Some(docs) = _docs {
        docs.iter()
            .map(|doc| index.schema().to_named_doc(doc))
            .collect_vec()
    } else {
        Vec::new()
    };

    match params.get_order() {
        Order::Asc => docs.reverse(),
        Order::Desc => {}
    }

    HttpResponse::Ok().json(docs)
}
