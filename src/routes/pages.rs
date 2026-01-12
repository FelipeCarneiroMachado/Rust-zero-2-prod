use actix_web::{HttpRequest, HttpResponse, Responder};
use actix_web::web::Html;

pub async fn index(req: HttpRequest) -> impl Responder {
    for file in std::fs::read_dir(".").unwrap() {
        println!("{:?}", file);
    }
    let body = std::fs::read_to_string("src/templates/index.html").unwrap();
    Html::new(body)
}