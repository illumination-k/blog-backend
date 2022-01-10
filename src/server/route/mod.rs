pub mod posts;

use actix_web::{get, Responder};

#[get("/")]
async fn hello() -> impl Responder {
    "Hello, Smark!"
}
