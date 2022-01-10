use actix_cors::Cors;
use actix_files;
use actix_web::{App, middleware, HttpServer};
use anyhow::Result;

use std::path::PathBuf;

use crate::text_engine::{index::read_or_build_index, schema::build_schema};

mod route;


#[actix_web::main]
pub async fn main(
    host: String,
    port: String,
    index_dir: PathBuf,
    static_dir: PathBuf,
    _cors_origin: Option<String>,
) -> Result<()> {
    let schema = build_schema();
    let index = read_or_build_index(schema, &index_dir, false)?;
    HttpServer::new(move || {
        if let Some(cors_origin) = _cors_origin.as_ref() {
            App::new()
                .data(index.clone())
                .wrap(middleware::Compress::default())
                .wrap(Cors::default().allowed_origin(cors_origin))
                .service(route::posts::get_posts)
                .service(route::posts::search::search_posts)
                .service(route::hello)
                .service(actix_files::Files::new("/public", &static_dir).show_files_listing())
        } else {
            App::new()
                .data(index.clone())
                .wrap(middleware::Compress::default())
                .wrap(Cors::default())
                .service(route::posts::get_posts)
                .service(route::posts::search::search_posts)
                .service(route::hello)
                .service(actix_files::Files::new("/public", &static_dir).show_files_listing())
        }
    })
    .bind(&format!("{}:{}", host, port))?
    .run()
    .await
    .expect("Error in build httpserver");
    Ok(())
}
