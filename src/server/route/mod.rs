pub mod openapi;
pub mod posts;
pub mod search;

use super::{CategoryList, TagList};

use actix_web::{get, web, HttpResponse, Responder};

#[get("/")]
async fn hello() -> impl Responder {
    "Hello, Smark!"
}

#[get("/tags")]
async fn tag_list(tags: web::Data<TagList>) -> impl Responder {
    let tags = tags.into_inner().0.clone();
    info!("tags: {:?}", tags);
    HttpResponse::Ok().json(tags)
}

#[get("/categories")]
async fn category_list(categories: web::Data<CategoryList>) -> impl Responder {
    let categories = categories.into_inner().0.clone();
    info!("categories: {:?}", categories);
    HttpResponse::Ok().json(categories)
}

#[cfg(test)]
mod test {
    use super::*;
    use actix_web::{
        dev::Service,
        http::StatusCode,
        test,
        web::{self, Bytes},
        App,
    };

    #[actix_web::test]
    async fn test_health_check() {
        let app = test::init_service(App::new().service(hello)).await;
        let req = test::TestRequest::get().uri("/").to_request();
        let resp = app.call(req).await.unwrap();

        assert_eq!(resp.response().status(), StatusCode::OK);
    }

    #[actix_web::test]
    async fn test_tags_categories() {
        let tags: Vec<String> = vec!["A", "B", "C"].iter().map(|x| x.to_string()).collect();
        let categories: Vec<String> = vec!["A", "B", "C"].iter().map(|x| x.to_string()).collect();
        let app = test::init_service(
            App::new()
                .app_data(web::Data::new(CategoryList(categories.clone())))
                .app_data(web::Data::new(TagList(tags.clone())))
                .service(tag_list)
                .service(category_list),
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
}
