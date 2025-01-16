use std::env;
use actix_web::{web, Responder, HttpResponse};
use reqwest::{Client, Method};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct FriendshipRequest {
    initiator_id: String,
    recipient_id: String,
    status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ApiResponse {
    pub message: String,
}

async fn make_friendship_request(
    client: &Client,
    path: &str,
    method: Method,
    data: &FriendshipRequest,
) -> HttpResponse {
    let endpoint = format!(
        "{}/friendships/{}",
        env::var("FRIENDSHIP_SERVICE_URL").unwrap_or_else(|_| "http://localhost:8086".to_string()),
        path
    );

    let request = match method {
        Method::POST => client.post(&endpoint).json(data),
        Method::PUT => client.put(&endpoint).json(data),
        _ => {
            return HttpResponse::InternalServerError().json(ApiResponse {
                message: "Unsupported HTTP method".to_string(),
            });
        }
    };

    match request.send().await {
        Ok(res) => match res.status() {
            reqwest::StatusCode::OK => HttpResponse::Ok().body(res.text().await.unwrap()),
            reqwest::StatusCode::BAD_REQUEST | reqwest::StatusCode::NOT_FOUND => {
                HttpResponse::BadRequest().json(
                    res.text().await.unwrap_or_else(|_| "Failed to read response".to_string()),
                )
            }
            _ => HttpResponse::InternalServerError().json(ApiResponse {
                message: "Failed to reach Friendship Service".to_string(),
            }),
        },
        Err(_) => HttpResponse::InternalServerError().json(ApiResponse {
            message: "Failed to reach Friendship Service".to_string(),
        }),
    }
}

pub async fn initiate_friendship(
    client: web::Data<Client>,
    data: web::Json<FriendshipRequest>,
) -> impl Responder {
    make_friendship_request(&client, "initiate", Method::POST, &data).await
}


pub async fn respond_to_friendship(client: web::Data<Client>, data: web::Json<FriendshipRequest>) -> impl Responder {
    make_friendship_request(&client, "respond", Method::PUT, &data).await
}