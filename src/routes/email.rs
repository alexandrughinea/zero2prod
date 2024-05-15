use actix_web::{HttpResponse, Responder};

pub async fn email() -> impl Responder {
    HttpResponse::Ok()
}
