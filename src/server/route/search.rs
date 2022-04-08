use actix_web::{get, web, HttpRequest, HttpResponse};
use itertools::Itertools;
use serde::Deserialize;
use std::ops::Deref;
use tantivy::{
    collector::TopDocs,
    query::{AllQuery, Query},
    Index,
};

use crate::text_engine::{
    query::search,
    schema::{FieldGetter, PostField},
};

#[derive(Debug, Deserialize)]
pub struct SearchQueryParams {
    query: Option<String>,
    limit: Option<usize>,
}

#[get("/search")]
async fn search_posts(index: web::Data<Index>, req: HttpRequest) -> HttpResponse {
    let index = index.into_inner();
    let schema = index.schema();
    let params = match web::Query::<SearchQueryParams>::from_query(req.query_string()) {
        Ok(p) => p,
        Err(e) => return HttpResponse::BadRequest().body(e.to_string()),
    };

    let fb = FieldGetter::new(&schema);
    let fields = [PostField::Title, PostField::Description, PostField::RawText]
        .into_iter()
        .map(|pf| fb.get_field(pf))
        .collect_vec();

    let limit = if let Some(limit) = params.limit.as_ref() {
        *limit
    } else {
        10
    };

    let docs = if let Some(query) = params.query.to_owned() {
        match search(&query.to_lowercase(), fields, limit, index.deref()) {
            Ok(docs) => docs,
            Err(e) => {
                error!("{:?}", e);
                return HttpResponse::InternalServerError().body("Internal Server Error");
            }
        }
        .iter()
        .flat_map(|doc| fb.to_json(doc))
        .collect_vec()
    } else {
        let q: Box<dyn Query> = Box::new(AllQuery {});
        let searcher = index.reader().expect("Not error in search api").searcher();

        searcher
            .search(&q, &TopDocs::with_limit(limit)).unwrap()
            .into_iter()
            .flat_map(|(_, doc_address)| searcher.doc(doc_address).ok() )
            .flat_map(|doc| fb.to_json(&doc).ok())
            .collect_vec()
    };
    HttpResponse::Ok().json(docs)
}

#[cfg(test)]
mod test_search {
    use crate::test_utility::*;
    use crate::text_engine::index::read_or_build_index;
    use crate::text_engine::schema::build_schema;

    use super::*;

    use actix_web::{dev::Service, http::StatusCode, test, web, App};
    use std::path::Path;
    use tempdir::TempDir;

    #[actix_web::test]
    async fn test_posts_search_empty() {
        let index_dir = "test/index";

        let schema = build_schema();
        let index = read_or_build_index(schema, &Path::new(index_dir), false).unwrap();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(index.clone()))
                .service(search_posts),
        )
        .await;
        let req = test::TestRequest::get().uri("/search").to_request();

        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.response().status(), StatusCode::OK);
        let _: Vec<PostResponse> = test::read_body_json(resp).await;
    }

    #[actix_web::test]
    async fn test_posts_seach_limit() {
        let temp_dir = TempDir::new(&format!(
            "temp_rand_index_{}",
            uuid::Uuid::new_v4().to_string()
        ))
        .unwrap();
        let (_, index) = build_random_posts_index(5, temp_dir.path()).unwrap();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(index.clone()))
                .service(search_posts),
        )
        .await;
        let req = test::TestRequest::get()
            .uri("/search?limit=2")
            .to_request();

        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.response().status(), StatusCode::OK);
        let resp_posts: Vec<PostResponse> = test::read_body_json(resp).await;
        assert_eq!(resp_posts.len(), 2);
    }
}
