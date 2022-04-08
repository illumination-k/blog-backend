use actix_cors::Cors;
use actix_web::{middleware, web, App, HttpServer};
use anyhow::Result;
use std::path::PathBuf;

use crate::text_engine::{
    index::read_or_build_index, query::get_tags_and_categories, schema::build_schema,
};

mod route;

pub struct CategoryList(Vec<String>);
pub struct TagList(Vec<String>);

#[actix_web::main]
pub async fn main(
    host: String,
    port: String,
    index_dir: PathBuf,
    static_dir: PathBuf,
    _cors_origin: Option<String>,
) -> Result<()> {
    eprintln!(
        "Index Dir: {}, Static Dir: {}",
        index_dir.display(),
        static_dir.display()
    );
    eprintln!("start running on {}:{}", host, port);

    let static_uri = "/public";
    eprintln!("static uri: {}", static_uri);

    let schema = build_schema();
    let index = read_or_build_index(schema, &index_dir, false)?;
    let (tags, categories) = get_tags_and_categories(&index)?;
    HttpServer::new(move || {
        if let Some(cors_origin) = _cors_origin.as_ref() {
            App::new()
                .app_data(web::Data::new(index.clone()))
                .app_data(web::Data::new(CategoryList(categories.clone())))
                .app_data(web::Data::new(TagList(tags.clone())))
                .wrap(middleware::Compress::default())
                .wrap(Cors::default().allowed_origin(cors_origin))
                .service(route::posts::get_post_by_id)
                .service(route::posts::get_posts)
                .service(route::posts::get_post_by_slug_and_lang)
                .service(route::search::search_posts)
                .service(route::hello)
                .service(route::tag_list)
                .service(route::category_list)
                .service(actix_files::Files::new(static_uri, &static_dir).show_files_listing())
        } else {
            App::new()
                .app_data(web::Data::new(index.clone()))
                .app_data(web::Data::new(CategoryList(categories.clone())))
                .app_data(web::Data::new(TagList(tags.clone())))
                .wrap(middleware::Compress::default())
                .wrap(Cors::default())
                .service(route::posts::get_post_by_id)
                .service(route::posts::get_posts)
                .service(route::posts::get_post_by_slug_and_lang)
                .service(route::search::search_posts)
                .service(route::hello)
                .service(route::tag_list)
                .service(route::category_list)
                .service(actix_files::Files::new(static_uri, &static_dir).show_files_listing())
        }
    })
    .bind(&format!("{}:{}", host, port))?
    .run()
    .await?;
    Ok(())
}

#[cfg(test)]
mod test {
    use super::*;
    use actix_web::{
        dev::Service,
        http::StatusCode,
        test,
        web::{self, Bytes},
    };
    use std::path::Path;
    use crate::test_utility::*;
    use tempdir::TempDir;

    #[actix_web::test]
    async fn test_tags_categories() {
        let tags: Vec<String> = vec!["A", "B", "C"].iter().map(|x| x.to_string()).collect();
        let categories: Vec<String> = vec!["A", "B", "C"].iter().map(|x| x.to_string()).collect();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(CategoryList(categories.clone())))
                .app_data(web::Data::new(TagList(tags.clone())))
                .service(route::tag_list)
                .service(route::category_list),
        )
        .await;

        for uri in &["/tags", "/categories"] {
            let req = test::TestRequest::get().uri(uri).to_request();
            let resp = app.call(req).await.unwrap();

            assert_eq!(resp.response().status(), StatusCode::OK);
            let v = test::read_body(resp).await;
            assert_eq!(v, Bytes::from_static(b"[\"A\",\"B\",\"C\"]"));
        }
    }

    #[actix_web::test]
    async fn test_posts_getall_empty() {
        let index_dir = "test/index";

        let schema = build_schema();
        let index = read_or_build_index(schema, &Path::new(index_dir), false).unwrap();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(index.clone()))
                .service(route::posts::get_posts),
        )
        .await;
        let req = test::TestRequest::get().uri("/posts").to_request();

        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.response().status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_posts_getall_lang() {
        let temp_dir = TempDir::new(&format!(
            "temp_rand_index_{}",
            uuid::Uuid::new_v4().to_string()
        )).unwrap();
        let (_, index) = build_random_posts_index(5, temp_dir.path()).unwrap();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(index.clone()))
                .service(route::posts::get_posts),
        )
        .await;
        let req = test::TestRequest::get().uri("/posts").param("lang", "en").to_request();
        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.response().status(), StatusCode::OK);
    }
    
    #[actix_web::test]
    async fn test_posts_search_empty() {
        let index_dir = "test/index";

        let schema = build_schema();
        let index = read_or_build_index(schema, &Path::new(index_dir), false).unwrap();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(index.clone()))
                .service(route::search::search_posts),
        )
        .await;
        let req = test::TestRequest::get().uri("/search").to_request();

        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.response().status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_get_by_uuid() {
        let temp_dir = TempDir::new(&format!(
            "temp_rand_index_{}",
            uuid::Uuid::new_v4().to_string()
        )).unwrap();
        let (posts, index) = build_random_posts_index(5, temp_dir.path()).unwrap();

        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(index.clone()))
                .service(route::posts::get_post_by_id),
        )
        .await;

        for post in posts.iter() {
            let path = format!("/post/uuid/{}", post.uuid());
            let req = test::TestRequest::get().uri(&path).to_request();
            let resp = app.call(req).await.unwrap();

            assert_eq!(resp.response().status(), StatusCode::OK);
        }
    }
}
