mod models;
use crate::models::{CreateProfileRequest};
use actix_web::{web, App, HttpServer};
use reqwest::Client;

mod services{
    pub mod friendship_service;
}
use crate::services::friendship_service::{initiate_friendship, respond_to_friendship};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let client = Client::new();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client.clone()))
            .route("/friendships/initiate", web::post().to(initiate_friendship))
            .route("/friendships/respond", web::put().to(respond_to_friendship))
    })
    .bind(("0.0.0.0", 8081))?
    .run()
    .await
}