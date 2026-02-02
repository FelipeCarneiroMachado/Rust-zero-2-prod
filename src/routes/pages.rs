use actix_web::Responder;
use actix_web::web::Html;

pub async fn index() -> impl Responder {
    for file in std::fs::read_dir(".").unwrap() {
        println!("{:?}", file);
    }
    let body = std::fs::read_to_string("src/templates/index.html").unwrap();
    Html::new(body)
}
