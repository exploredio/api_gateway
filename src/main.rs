use actix_web::{web, App, HttpServer, HttpResponse, Result, HttpRequest};
use reqwest::{Client, Response, StatusCode, Error};
use serde::Deserialize;
use std::collections::HashMap;
use std::env;
use actix_web::http::Method;
use serde_json::Value;

#[derive(Deserialize)]
struct ServicePath {
    service_name: String,
    extra_path: Option<String>,
}

async fn route_to_service(
    req: HttpRequest,
    path: web::Path<ServicePath>,
    body: Option<web::Json<Value>>,
    client: web::Data<Client>,
) -> Result<HttpResponse> {
    let service_urls = vec![
        ("profiles".to_string(),
         env::var("PROFILE_SERVICE_URL").unwrap_or_else(|_| "http://localhost:8082".to_string()
         )),
        ("friendships".to_string(),
         env::var("FRIENDSHIP_SERVICE_URL").unwrap_or_else(|_| "http://localhost:8086".to_string()
         )),
    ]
        .into_iter()
        .collect::<HashMap<String, String>>();

    // Extract the service name and any extra path
    let service_name = &path.service_name;
    let extra_path = path.extra_path.clone().unwrap_or_default();

    // Check if the service exists in the map
    if let Some(service_url) = service_urls.get(service_name) {
        let url = if extra_path.is_empty() {
            format!("{}/{}", service_url, service_name)
        } else {
            format!("{}/{}/{}", service_url, service_name, extra_path)
        };

        match req.method().clone() {
            Method::GET => {
                let response = client.get(&url).send().await;
                handle_service_response(response).await
            }
            Method::POST => {
                let response = match body {
                    Some(json_body) => {
                        client.post(&url).json(&json_body.into_inner()).send().await
                    }
                    None => {
                        client.post(&url).send().await
                    }
                };
                handle_service_response(response).await
            }
            Method::PUT => {
                let response = match body {
                    Some(json_body) => {
                        client.put(&url).json(&json_body.into_inner()).send().await
                    }
                    None => {
                        client.put(&url).send().await
                    }
                };
                handle_service_response(response).await
            }
            Method::DELETE => {
                let response = client.delete(&url).send().await;
                handle_service_response(response).await
            }
            _ => Ok(HttpResponse::MethodNotAllowed().body("Unsupported HTTP method")),
        }
    } else {
        Ok(HttpResponse::NotFound().body("Service not found"))
    }
}

async fn handle_service_response(response: Result<Response, Error>) -> Result<HttpResponse> {
    match response {
        Ok(resp) => {
            let status = resp.status();

            let content_type = resp
                .headers()
                .get("Content-Type")
                .and_then(|ct| ct.to_str().ok())
                .unwrap_or_default();

            if content_type.contains("application/json") {
                let body = match resp.json::<Value>().await {
                    Ok(json_body) => json_body,
                    Err(err) => {
                        return Ok(HttpResponse::InternalServerError().body(err.to_string()));
                    }
                };

                match status {
                    StatusCode::OK => Ok(HttpResponse::Ok().json(body)),
                    StatusCode::NOT_FOUND => Ok(HttpResponse::NotFound().json(body)),
                    StatusCode::CREATED => Ok(HttpResponse::Created().json(body)),
                    StatusCode::BAD_REQUEST => Ok(HttpResponse::BadRequest().json(body)),
                    _ => Ok(HttpResponse::InternalServerError().body("Unhandled HTTP status code")),
                }
            } else {
                let body = resp.text().await.unwrap_or_default();

                match status {
                    StatusCode::OK => Ok(HttpResponse::Ok().json(body)),
                    StatusCode::NOT_FOUND => Ok(HttpResponse::NotFound().json(body)),
                    StatusCode::CREATED => Ok(HttpResponse::Created().json(body)),
                    StatusCode::BAD_REQUEST => Ok(HttpResponse::BadRequest().json(body)),
                    _ => Ok(HttpResponse::InternalServerError().body("Unhandled HTTP status code")),
                }
            }
        }
        Err(_) => {
            Ok(HttpResponse::InternalServerError().body("Failed to reach service"))
        }
    }
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let client = Client::new();

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(client.clone()))
            .route("/{service_name}", web::to(route_to_service)) // Dynamic routing for native service names
            .route("/{service_name}/{extra_path:.*}", web::to(route_to_service)) // Dynamic routing for extra paths after service name
    })
        .bind("0.0.0.0:8081")?
        .run()
        .await
}
