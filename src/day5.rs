use std::f32::consts::E;

use axum::{http::HeaderMap, http::StatusCode, response::IntoResponse, body::{Body, to_bytes}};
use serde::Deserialize;
use toml::de::Deserializer;
use cargo_manifest::Manifest;

#[derive(Deserialize, Debug)]
struct Package {
    package: PackageInfo,
}

#[derive(Deserialize, Debug)]
struct ValuePackege {
    package: toml::Value,
}

#[derive(Deserialize, Debug)]
struct PackageInfo {
    name: String,
    authors: Vec<String>,
    keywords: Vec<String>,
    metadata: Metadata,
}

#[derive(Deserialize, Debug)]
struct Metadata {
    orders: Vec<Order>,
}

#[derive(Deserialize, Debug)]
struct Order {
    item: String,
    quantity: u32,
}

#[axum::debug_handler]
pub async fn return_manifest(headers: HeaderMap, body: Body) -> impl IntoResponse {
    match headers.get("Content-Type").map(|v| v.to_str().unwrap()) {
        Some("application/toml") => {
            let content_size:usize = headers.get("Content-Length").expect("No Content-Length header").to_str().unwrap().parse().expect("Invalid Content-Length header");
            let bytes = to_bytes(body, content_size).await.expect("Invalid body");
            let toml_str = String::from_utf8(bytes.to_vec()).expect("Invalid body");

            if let Err(e) = toml::from_str::<Manifest>(&toml_str) {
                return (StatusCode::BAD_REQUEST, "Invalid manifest".to_string())
            }

            let de = Deserializer::new(&toml_str);
            let package: Package = match Package::deserialize(de) {
                Ok(package) => package,
                Err(e) => {
                    println!("Invalid manifest: {:?}", e);
                    return (StatusCode::NO_CONTENT, "No order found".to_string())
                }
            };
            
            println!("{:?}", package);
            if let Some(order) = package.package.metadata.orders.first() {
                (StatusCode::OK, package.package.metadata.orders.iter().map(|order| format!("{}: {}",order.item,  order.quantity)).collect::<Vec<String>>().join("\n"))
            } else {
                (StatusCode::NO_CONTENT, "No order found".to_string())
            }

        },
        Some(_) => {
            println!("Invalid Content-Type header");
            (StatusCode::UNSUPPORTED_MEDIA_TYPE, "".to_string())
        },
        None => {
            println!("No Content-Type header");
            (StatusCode::OK, "No Content-Type header".to_string())
        }
    }

}
