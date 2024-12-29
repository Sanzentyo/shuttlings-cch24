use axum::{http::HeaderMap, http::StatusCode, response::IntoResponse, body::{Body, to_bytes}};
use serde::Deserialize;
use toml;
use cargo_manifest::Manifest;
use serde_json;


#[derive(Deserialize, Debug)]
struct Order {
    item: String,
    quantity: u32,
}

impl std::fmt::Display for Order {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.item.to_string(), self.quantity)
    }
}

#[axum::debug_handler]
pub async fn return_manifest(headers: HeaderMap, body: Body) -> impl IntoResponse {
    match headers.get("Content-Type").map(|v| v.to_str().unwrap()) {
        Some("application/toml") => {
            let content_size:usize = headers.get("Content-Length").expect("No Content-Length header").to_str().unwrap().parse().expect("Invalid Content-Length header");
            let bytes = to_bytes(body, content_size).await.expect("Invalid body");
            let toml_str = String::from_utf8(bytes.to_vec()).expect("Invalid body");

            let toml = if let Ok(toml) = toml::from_str::<Manifest>(&toml_str) {
                toml
            } else {
                return (StatusCode::BAD_REQUEST, "Invalid manifest".to_string())
            };

            let packege = if let Some(packege) = toml.package{
                packege
            } else {
                return (StatusCode::NO_CONTENT, "No package found".to_string())
            };

            let metadata: toml::Value = if let Some(metadata) = packege.metadata {
                metadata
            } else {
                return (StatusCode::NO_CONTENT, "No metadata found".to_string())
            };
            
            let order_value = if let Some(order) = metadata.get("orders") {
                order
            } else {
                return (StatusCode::NO_CONTENT, "No order found".to_string())
            };

            let mut orders: Vec<Order> = Vec::new();
            for order in order_value.as_array().unwrap() {
                let item = if let Some(item) = order.get("item") {
                    let item = match item {
                        toml::Value::String(item) => item,
                        _ => continue,
                    };
                    item.to_owned()
                } else {
                    continue;
                };
                let quantity:u32 = {
                    let quantity_str = if let Some(quantity) = order.get("quantity") {
                        quantity.to_string()
                    } else {
                        continue;
                    };
                    if let Ok(quantity) = quantity_str.parse::<u32>() {
                        quantity
                    } else {
                        continue;
                    }
                };
                orders.push(Order{item, quantity});
            }

            if orders.len() == 0 {
                return (StatusCode::NO_CONTENT, "No order found".to_string())
            }

            let result: Vec<String> = orders.iter().map(|order| order.to_string()).collect();
            println!("result: {:?}", result);

            (StatusCode::OK, result.join("\n"))

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
