use actix_web::{get, web, HttpRequest, HttpResponse};
use serde::{Deserialize, Serialize};
use std::ops::Deref;
use tantivy::{
    collector::Count,
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
            return HttpResponse::NotFound().body(format!(
                "slug: '{}' lang: '{}' is not found!",
                &params.slug, &lang
            ));
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
        let field_iter = [PostField::Lang, PostField::Category, PostField::Tags].iter();

        [
            self.lang.to_owned(),
            self.category.to_owned(),
            self.tag.to_owned(),
        ]
        .into_iter()
        .zip(field_iter)
        .flat_map(|(val, &pf)| {
            if let Some(val) = val {
                let field = fb.get_field(pf);
                let q: Box<dyn Query> = Box::new(TermQuery::new(
                    Term::from_field_text(field, &val),
                    IndexRecordOption::Basic,
                ));
                Some((Occur::Must, q))
            } else {
                None
            }
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

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct Counter {
    count: usize,
}

#[get("/post/count")]
async fn count_posts(req: HttpRequest, index: web::Data<Index>) -> HttpResponse {
    let index = index.into_inner();
    let schema = index.schema();
    let fb = FieldGetter::new(&schema);
    let params = match web::Query::<GetPostsQueryParams>::from_query(req.query_string()) {
        Ok(p) => p,
        Err(e) => return HttpResponse::BadRequest().body(e.to_string()),
    };

    let queries = params.to_queries(&fb);
    let query: Box<dyn Query> = if queries.is_empty() {
        Box::new(AllQuery {})
    } else {
        Box::new(BooleanQuery::new(queries))
    };

    let counter = Count {};
    let searcher = index.reader().expect("Not error here").searcher();
    let count = searcher.search(&query, &counter);

    if let Ok(count) = count {
        HttpResponse::Ok().json(Counter { count })
    } else {
        HttpResponse::InternalServerError().body("Internal server error\n")
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
            let empty: Vec<String> = Vec::new();
            return HttpResponse::Ok().json(empty);
        }
    };

    let mut docs = if let Some(docs) = _docs {
        docs.iter().flat_map(|doc| fb.to_json(doc).ok()).collect()
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
    use crate::{test_utility::*, text_engine::query::put};
    use actix_web::{dev::Service, http::StatusCode, test, web, App};
    use anyhow::Result;
    use tempdir::TempDir;
    use urlencoding::encode;

    fn uuid_tempdir() -> TempDir {
        TempDir::new(&format!(
            "temp_rand_index_{}",
            uuid::Uuid::new_v4().to_string()
        ))
        .unwrap()
    }

    async fn test_count_and_get_posts(
        query_params: &str,
        index: &Index,
    ) -> Result<(Counter, Vec<PostResponse>)> {
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(index.clone()))
                .service(get_posts)
                .service(count_posts),
        )
        .await;

        let posts_resp = {
            let req = test::TestRequest::get()
                .uri(&format!("/posts{}", query_params))
                .to_request();
            let resp = app.call(req).await.unwrap();
            assert_eq!(resp.response().status(), StatusCode::OK);

            test::read_body_json(resp).await
        };

        let counter = {
            let req = test::TestRequest::get()
                .uri(&format!("/post/count{}", query_params))
                .to_request();
            let resp = app.call(req).await.unwrap();
            assert_eq!(resp.response().status(), StatusCode::OK);

            test::read_body_json(resp).await
        };

        Ok((counter, posts_resp))
    }

    #[actix_web::test]
    async fn test_posts_count_get_no_params() {
        let temp_dir = TempDir::new(&format!(
            "temp_rand_index_{}",
            uuid::Uuid::new_v4().to_string()
        ))
        .unwrap();
        let post_num = 5;
        let (_, index) = build_random_posts_index(post_num, temp_dir.path()).unwrap();

        let (count, posts) = test_count_and_get_posts("", &index).await.unwrap();

        assert_eq!(posts.len(), post_num);
        assert_eq!(count.count, post_num);
    }

    #[actix_web::test]
    async fn test_posts_count_get_lang() {
        let temp_dir = TempDir::new(&format!(
            "temp_rand_index_{}",
            uuid::Uuid::new_v4().to_string()
        ))
        .unwrap();
        let post_num = 5;
        let (posts, index) = build_random_posts_index(post_num, temp_dir.path()).unwrap();
        let lang_posts_num = posts.iter().filter(|x| x.lang() == Lang::En).count();
        let (count, posts) =
            test_count_and_get_posts(&format!("?lang={}", Lang::En.as_str()), &index)
                .await
                .unwrap();

        assert_eq!(posts.len(), lang_posts_num);
        assert_eq!(count.count, lang_posts_num);
    }

    #[actix_web::test]
    async fn test_posts_count_get_categories() -> Result<()> {
        let temp_dir = TempDir::new(&format!(
            "temp_rand_index_{}",
            uuid::Uuid::new_v4().to_string()
        ))?;

        let (posts, index) = build_random_posts_index(5, temp_dir.path()).unwrap();
        let category = posts[0].category();
        let category_post_num = posts.iter().filter(|p| p.category() == category).count();
        let (count, posts) =
            test_count_and_get_posts(&format!("?category={}", encode(&category)), &index)
                .await
                .unwrap();

        for post in posts.iter() {
            assert_eq!(post.category, category);
        }

        assert_eq!(posts.len(), category_post_num);
        assert_eq!(count.count, category_post_num);

        Ok(())
    }

    #[actix_web::test]
    async fn test_posts_count_get_tags() {
        let temp_dir = uuid_tempdir();

        let (mut posts, index) = build_random_posts_index(5, temp_dir.path()).unwrap();
        *posts[0].tags_mut() = Some(vec!["test0".to_string(), "test1".to_string()]);
        *posts[1].tags_mut() = Some(vec![
            "test1".to_string(),
            "github-actions".to_string(),
            "next.js".to_string(),
        ]);
        let mut index_writer = index.writer(100000000).unwrap();
        put(&posts[0], &index, &mut index_writer, false).unwrap();
        put(&posts[1], &index, &mut index_writer, false).unwrap();

        let (count, _) = test_count_and_get_posts("?tag=test1", &index)
            .await
            .unwrap();

        assert_eq!(count.count, 2);

        let (count, _) = test_count_and_get_posts("?tag=github-actions", &index)
            .await
            .unwrap();

        assert_eq!(count.count, 1);

        let (count, _) = test_count_and_get_posts("?tag=next.js", &index)
            .await
            .unwrap();

        assert_eq!(count.count, 1);
    }

    #[actix_web::test]
    async fn test_posts_count_get_not_found() {
        let temp_dir = TempDir::new(&format!(
            "temp_rand_index_{}",
            uuid::Uuid::new_v4().to_string()
        ))
        .unwrap();
        let (_, index) = build_random_posts_index(5, temp_dir.path()).unwrap();

        let (count, posts) = test_count_and_get_posts(
            "?category=a&tag=a&lang=en&order_by=created_at&order=desc",
            &index,
        )
        .await
        .unwrap();
        assert!(posts.is_empty());
        assert_eq!(count.count, 0);
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

    #[actix_web::test]
    async fn test_get_by_uuid_not_found() {
        let temp_dir = TempDir::new(&format!(
            "temp_rand_index_{}",
            uuid::Uuid::new_v4().to_string()
        ))
        .unwrap();

        let (_, index) = build_random_posts_index(5, temp_dir.path()).unwrap();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(index.clone()))
                .service(get_post_by_id),
        )
        .await;
        let req = test::TestRequest::get().uri("/post/uuid/a").to_request();
        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.response().status(), StatusCode::NOT_FOUND);
    }
}
