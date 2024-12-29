use axum::{http::HeaderMap, http::StatusCode, response::IntoResponse, body::{Body, to_bytes}};
use serde::Deserialize;
use toml;
use cargo_manifest::Manifest;
use serde_yml;
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
                return (StatusCode::BAD_REQUEST, "Invalid manifest".to_string())
            };

            if let Some(keywords) = packege.keywords {
                match keywords {
                    cargo_manifest::MaybeInherited::Local(keywords) => {
                        if !keywords.contains(&"Christmas 2024".to_string()) {
                            return (StatusCode::BAD_REQUEST, "Magic keyword not provided".to_string())
                        }
                    },
                    _ => {
                        return (StatusCode::BAD_REQUEST, "Magic keyword not provided".to_string())
                    }
                }
            } else {
                return (StatusCode::BAD_REQUEST, "Magic keyword not provided".to_string())
            }



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

            (StatusCode::OK, result.join("\n"))

        },
        Some("application/yaml") => {
            let content_size:usize = headers.get("Content-Length").expect("No Content-Length header").to_str().unwrap().parse().expect("Invalid Content-Length header");
            let bytes = to_bytes(body, content_size).await.expect("Invalid body");
            let yaml_str = String::from_utf8(bytes.to_vec()).expect("Invalid body");

            // YAMLを直接Manifestに変換
            let manifest: Manifest = if let Ok(manifest) = serde_yml::from_str(&yaml_str) {
                manifest
            } else {
                return (StatusCode::BAD_REQUEST, "Invalid manifest".to_string());
            };

            // Cargoマニフェストの検証
            let package = if let Some(package) = manifest.package {
                package
            } else {
                return (StatusCode::BAD_REQUEST, "Invalid manifest".to_string());
            };

            if let Some(keywords) = package.keywords {
                match keywords {
                    cargo_manifest::MaybeInherited::Local(keywords) => {
                        if !keywords.contains(&"Christmas 2024".to_string()) {
                            return (StatusCode::BAD_REQUEST, "Magic keyword not provided".to_string())
                        }
                    },
                    _ => return (StatusCode::BAD_REQUEST, "Magic keyword not provided".to_string())
                }
            } else {
                return (StatusCode::BAD_REQUEST, "Magic keyword not provided".to_string());
            }

            let metadata: serde_yml::Value = if let Some(metadata) = package.metadata {
                serde_yml::from_value(serde_yml::to_value(metadata).unwrap()).unwrap()
            } else {
                return (StatusCode::NO_CONTENT, "No metadata found".to_string());
            };

            let orders = if let Some(orders) = metadata.get("orders") {
                if let Some(orders) = orders.as_sequence() {
                    orders
                } else {
                    return (StatusCode::NO_CONTENT, "Invalid orders format".to_string());
                }
            } else {
                return (StatusCode::NO_CONTENT, "No order found".to_string());
            };

            let mut parsed_orders: Vec<Order> = Vec::new();
            for order in orders {
                let item = if let Some(item) = order.get("item").and_then(|i| i.as_str()) {
                    item.to_string()
                } else {
                    continue;
                };
                
                let quantity = if let Some(quantity) = order.get("quantity").and_then(|q| q.as_u64()) {
                    quantity as u32
                } else {
                    continue;
                };

                parsed_orders.push(Order { item, quantity });
            }

            if parsed_orders.is_empty() {
                return (StatusCode::NO_CONTENT, "No order found".to_string());
            }

            let result: Vec<String> = parsed_orders.iter().map(|order| order.to_string()).collect();
            (StatusCode::OK, result.join("\n"))
        },
        Some("application/json") => {
            let content_size:usize = headers.get("Content-Length").expect("No Content-Length header").to_str().unwrap().parse().expect("Invalid Content-Length header");
            let bytes = to_bytes(body, content_size).await.expect("Invalid body");
            let json_str = String::from_utf8(bytes.to_vec()).expect("Invalid body");

            // JSONを直接Manifestに変換
            let manifest: Manifest = if let Ok(manifest) = serde_json::from_str(&json_str) {
                manifest
            } else {
                return (StatusCode::BAD_REQUEST, "Invalid manifest".to_string());
            };

            // Cargoマニフェストの検証
            let package = if let Some(package) = manifest.package {
                package
            } else {
                return (StatusCode::BAD_REQUEST, "Invalid manifest".to_string());
            };

            if let Some(keywords) = package.keywords {
                match keywords {
                    cargo_manifest::MaybeInherited::Local(keywords) => {
                        if !keywords.contains(&"Christmas 2024".to_string()) {
                            return (StatusCode::BAD_REQUEST, "Magic keyword not provided".to_string())
                        }
                    },
                    _ => return (StatusCode::BAD_REQUEST, "Magic keyword not provided".to_string())
                }
            } else {
                return (StatusCode::BAD_REQUEST, "Magic keyword not provided".to_string());
            }

            // 以降は既存のJSONパース処理を継続
            let json: serde_json::Value = serde_json::from_str(&json_str).expect("Invalid JSON");
            let package = if let Some(package) = json.get("package") {
                package
            } else {
                return (StatusCode::BAD_REQUEST, "Invalid manifest".to_string());
            };

            // Check keywords
            let keywords = if let Some(keywords) = package.get("keywords") {
                if let Some(keywords) = keywords.as_array() {
                    keywords
                } else {
                    return (StatusCode::BAD_REQUEST, "Invalid keywords format".to_string());
                }
            } else {
                return (StatusCode::BAD_REQUEST, "Magic keyword not provided".to_string());
            };

            if !keywords.iter().any(|k| k.as_str() == Some("Christmas 2024")) {
                return (StatusCode::BAD_REQUEST, "Magic keyword not provided".to_string());
            }

            // Get metadata and orders
            let metadata = if let Some(metadata) = package.get("metadata") {
                metadata
            } else {
                return (StatusCode::NO_CONTENT, "No metadata found".to_string());
            };

            let orders = if let Some(orders) = metadata.get("orders") {
                if let Some(orders) = orders.as_array() {
                    orders
                } else {
                    return (StatusCode::NO_CONTENT, "Invalid orders format".to_string());
                }
            } else {
                return (StatusCode::NO_CONTENT, "No order found".to_string());
            };

            let mut parsed_orders: Vec<Order> = Vec::new();
            for order in orders {
                let item = if let Some(item) = order.get("item").and_then(|i| i.as_str()) {
                    item.to_string()
                } else {
                    continue;
                };
                
                let quantity = if let Some(quantity) = order.get("quantity").and_then(|q| q.as_u64()) {
                    quantity as u32
                } else {
                    continue;
                };

                parsed_orders.push(Order { item, quantity });
            }

            if parsed_orders.is_empty() {
                return (StatusCode::NO_CONTENT, "No order found".to_string());
            }

            let result: Vec<String> = parsed_orders.iter().map(|order| order.to_string()).collect();
            (StatusCode::OK, result.join("\n"))
        },
        Some(_) => {
            println!("Unsupported Content-Type");
            (StatusCode::UNSUPPORTED_MEDIA_TYPE, "".to_string())
        },
        None => {
            println!("No Content-Type header");
            (StatusCode::OK, "No Content-Type header".to_string())
        }
    }

}
