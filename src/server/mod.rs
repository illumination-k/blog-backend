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
                .service(route::openapi::get_openapi_schema)
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
                .service(route::openapi::get_openapi_schema)
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
