use actix_web::{App, HttpServer};
use anyhow::Result;

use std::path::PathBuf;

use crate::text_engine::{index::read_or_build_index, schema::build_schema};

mod route;

#[actix_web::main]
pub async fn main(host: String, port: String, index_dir: PathBuf) -> Result<()> {
    let schema = build_schema();
    let index = read_or_build_index(schema, &index_dir, false)?;
    HttpServer::new(move || {
        App::new()
            .data(index.clone())
            .service(route::posts::get_posts)
            .service(route::posts::search::search_posts)
            .service(route::hello)
    })
    .bind(&format!("{}:{}", host, port))?
    .run()
    .await
    .expect("Error in build httpserver");
    Ok(())
}
