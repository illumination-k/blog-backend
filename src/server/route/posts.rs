use actix_web::{get, web, HttpRequest, HttpResponse};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use tantivy::{
    query::{AllQuery, BooleanQuery, Occur, Query, TermQuery},
    schema::IndexRecordOption,
    Index, Term,
};

use crate::{
    posts::Lang,
    text_engine::{
        query::{get_all, get_by_slug_with_lang, get_by_uuid},
        schema::{FieldGetter, PostField},
    },
};

use crate::datetime;

#[get("/post/uuid/{uuid}")]
async fn get_post_by_id(index: web::Data<Index>, uuid: web::Path<String>) -> HttpResponse {
    let schema = index.schema();
    let fg = FieldGetter::new(&schema);
    match get_by_uuid(&uuid.to_owned(), index.into_inner().deref()) {
        Ok(doc) => HttpResponse::Ok().json(fg.to_json(&doc).unwrap()),
        Err(e) => HttpResponse::NotFound().body(e.to_string()),
    }
}

#[derive(Debug, Deserialize)]
pub struct GetSlugParams {
    slug: String,
    lang: Option<String>,
}

#[get("/post/slug")]
async fn get_post_by_slug_and_lang(index: web::Data<Index>, req: HttpRequest) -> HttpResponse {
    let index = index.into_inner();
    let schema = index.schema();
    let fb = FieldGetter::new(&schema);
    let params = match web::Query::<GetSlugParams>::from_query(req.query_string()) {
        Ok(p) => p,
        Err(e) => return HttpResponse::BadRequest().body(e.to_string()),
    };

    let lang = if let Some(lang) = params.lang.as_ref() {
        lang.to_owned()
    } else {
        Lang::Ja.to_string()
    };

    let doc = match get_by_slug_with_lang(&params.slug, &lang, &index) {
        Ok(_doc) => match fb.to_json(&_doc) {
            Ok(doc) => doc,
            Err(e) => {
                error!("{:?}", e);
                return HttpResponse::InternalServerError().body("Internal Server Error");
            }
        },
        Err(e) => {
            error!("{:?}", e);
            return HttpResponse::InternalServerError().body("Internal Server Error");
        }
    };

    HttpResponse::Ok().json(doc)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Order {
    Asc,
    Desc,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GetPostsQueryParams {
    lang: Option<String>,
    category: Option<String>,
    tag: Option<String>,
    order_by: Option<datetime::OrderBy>,
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

    pub fn order_by(&self) -> Option<datetime::OrderBy> {
        self.order_by.to_owned()
    }

    pub fn get_order(&self) -> Order {
        self.order.to_owned().unwrap_or(Order::Desc)
    }
}

#[get("/posts")]
async fn get_posts(req: HttpRequest, index: web::Data<Index>) -> HttpResponse {
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
            .flat_map(|doc| fb.to_json(doc).ok())
            .collect_vec()
    } else {
        Vec::new()
    };
    info!("{:?}", docs);
    match params.get_order() {
        Order::Asc => docs.reverse(),
        Order::Desc => {}
    }

    HttpResponse::Ok().json(docs)
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::test_utility::*;
    use actix_web::{dev::Service, http::StatusCode, test, web, App};
    use tempdir::TempDir;

    #[actix_web::test]
    async fn test_posts_getall_empty() {
        let temp_dir = TempDir::new(&format!(
            "temp_rand_index_{}",
            uuid::Uuid::new_v4().to_string()
        ))
        .unwrap();
        let post_num = 5;
        let (_, index) = build_random_posts_index(post_num, temp_dir.path()).unwrap();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(index.clone()))
                .service(get_posts),
        )
        .await;
        let req = test::TestRequest::get().uri("/posts").to_request();

        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.response().status(), StatusCode::OK);
        let resp_posts: Vec<PostResponse> = test::read_body_json(resp).await;
        assert_eq!(resp_posts.len(), post_num);
    }

    #[actix_web::test]
    async fn test_posts_getall_lang() {
        let temp_dir = TempDir::new(&format!(
            "temp_rand_index_{}",
            uuid::Uuid::new_v4().to_string()
        ))
        .unwrap();
        let post_num = 5;
        let (posts, index) = build_random_posts_index(post_num, temp_dir.path()).unwrap();
        let lang_posts_num = posts.iter().filter(|x| x.lang() == Lang::En).count();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(index.clone()))
                .service(get_posts),
        )
        .await;
        let req = test::TestRequest::get().uri("/posts?lang=en").to_request();
        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.response().status(), StatusCode::OK);
        let resp_posts: Vec<PostResponse> = test::read_body_json(resp).await;
        assert_eq!(lang_posts_num, resp_posts.len());
    }

    #[actix_web::test]
    async fn test_posts_getall_fullparams() {
        let temp_dir = TempDir::new(&format!(
            "temp_rand_index_{}",
            uuid::Uuid::new_v4().to_string()
        ))
        .unwrap();
        let (_, index) = build_random_posts_index(5, temp_dir.path()).unwrap();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(index.clone()))
                .service(get_posts),
        )
        .await;
        let req = test::TestRequest::get()
            .uri("/posts?category=a&tag=a&lang=en&order_by=created_at&order=desc")
            .to_request();
        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.response().status(), StatusCode::OK);
        let _: Vec<PostResponse> = test::read_body_json(resp).await;
    }

    #[actix_web::test]
    async fn test_get_by_uuid() {
        let temp_dir = TempDir::new(&format!(
            "temp_rand_index_{}",
            uuid::Uuid::new_v4().to_string()
        ))
        .unwrap();
        let (posts, index) = build_random_posts_index(5, temp_dir.path()).unwrap();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(index.clone()))
                .service(get_post_by_id),
        )
        .await;

        for post in posts.iter() {
            let path = format!("/post/uuid/{}", post.uuid());
            let req = test::TestRequest::get().uri(&path).to_request();
            let resp = app.call(req).await.unwrap();

            assert_eq!(resp.response().status(), StatusCode::OK);
            let p: PostResponse = test::read_body_json(resp).await;
            assert_eq!(p.uuid, post.uuid())
        }
    }
}
