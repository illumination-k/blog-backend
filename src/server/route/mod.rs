pub mod posts;

use super::{CategoryList, TagList};

use actix_web::{get, web, HttpResponse, Responder};

#[get("/")]
async fn hello() -> impl Responder {
    "Hello, Smark!"
}

#[get("/tags")]
async fn tag_list(tags: web::Data<TagList>) -> impl Responder {
    let tags = tags.into_inner().0.clone();
    HttpResponse::Ok().json(tags)
}

#[get("/categories")]
async fn category_list(categories: web::Data<CategoryList>) -> impl Responder {
    let categories = categories.into_inner().0.clone();
    HttpResponse::Ok().json(categories)
}
