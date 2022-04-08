use actix_web::{get, HttpResponse};

#[get("/openapi.yml")]
async fn get_openapi_schema() -> HttpResponse {
    let openapi_yaml = include_str!("../../../openapi.yml");
    HttpResponse::Ok().body(openapi_yaml)
}

#[cfg(test)]
mod test {
    use super::*;
    use actix_web::{dev::Service, http::StatusCode, test, web::Bytes, App};

    #[actix_web::test]
    async fn test_openapi() {
        let app = test::init_service(App::new().service(get_openapi_schema)).await;
        let req = test::TestRequest::get().uri("/openapi.yml").to_request();
        let resp = app.call(req).await.unwrap();
        assert_eq!(resp.response().status(), StatusCode::OK);

        let openapi_yaml = include_str!("../../../openapi.yml");
        let v = test::read_body(resp).await;
        assert_eq!(v, Bytes::from_static(openapi_yaml.as_bytes()));
    }
}
