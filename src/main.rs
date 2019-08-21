use actix_web::{http,middleware, get, App, HttpResponse, HttpServer, Responder};

//Static files
use std::collections::HashMap;
include!(concat!(env!("OUT_DIR"), "/generated.rs"));

#[get("/")]
fn index3() -> impl Responder {
    HttpResponse::Found()
        .header(http::header::LOCATION, "/static/index.html")
        .finish()
        .into_body()
}
fn main() {
    HttpServer::new(|| {
        let generated = generate();
        App::new()
            .wrap(middleware::NormalizePath)
            .service(index3)
            .service(actix_web_static_files::ResourceFiles::new(
                "/static", generated,
            ))
    })
    .bind("0.0.0.0:8080")
    .unwrap()
    .run()
    .unwrap();
}
