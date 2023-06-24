use std::sync::Mutex;
use crate::drive::Drive;

use super::models::DriveParams;

use actix_web::{get, post, HttpResponse, web};


#[get("/")]
async fn index() -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html; charset=utf-8")
        .body(include_str!("../../templates/index.html"))
}

#[post("/drive")]
async fn drive_handler(drive_data: web::Data<Mutex<Drive>>, web::Query(params): web::Query<DriveParams>) -> HttpResponse {
    let mut drive_mutex = drive_data.lock().unwrap();
    drive_mutex.move_robot(&params.direction, &params.speed);
    HttpResponse::Ok().body("I've moved\n")
}

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(index);
    cfg.service(drive_handler);
}