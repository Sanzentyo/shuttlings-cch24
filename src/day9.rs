use axum::{
    body::{Body, to_bytes}, extract::State, http::{HeaderMap, StatusCode}, response::IntoResponse  
};
use leaky_bucket::RateLimiter;
use std::sync::{Mutex, Arc};
use std::time::Duration;
use serde::{Deserialize, Serialize};


pub const MAX_MILK: usize = 5;

#[derive(Debug, Clone)]
pub struct MilkState {
    pub limiter: Arc<Mutex<RateLimiter>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LiterRequest {
    pub liters: f32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct GallonsRequest {
    pub gallons: f32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct LitresRequest {
    pub litres: f32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct PintsRequest {
    pub pints: f32,
}

pub async fn milk_and_cookies(
    State(milk_state): State<MilkState>,
    headers: HeaderMap,
    body: Body,
) -> impl IntoResponse {
    match headers.get("Content-Type").map(|v| v.to_str().unwrap()) {
        Some("application/json") => {
            if !milk_state.limiter.lock().unwrap().try_acquire(1) {
                return (StatusCode::TOO_MANY_REQUESTS, "No milk available\n".to_string());
            }

            let bytes = to_bytes(body, 1024).await.unwrap();
            match (
                serde_json::from_slice::<LiterRequest>(&bytes),
                serde_json::from_slice::<GallonsRequest>(&bytes),
                serde_json::from_slice::<LitresRequest>(&bytes),
                serde_json::from_slice::<PintsRequest>(&bytes),
            ) {
                (Ok(LiterRequest { liters }), Err(_), Err(_), Err(_)) => {
                    let gallons = liters * 0.264172052;
                    let gallons_request = GallonsRequest { gallons };
                    let response = serde_json::to_string(&gallons_request).unwrap();
                    (StatusCode::OK, response)
                }
                (Err(_), Ok(GallonsRequest { gallons }), Err(_), Err(_)) => {
                    let liters = gallons * 3.785411784;
                    let liters_request = LiterRequest { liters };
                    let response = serde_json::to_string(&liters_request).unwrap();
                    (StatusCode::OK, response)
                }
                (Err(_), Err(_), Ok(LitresRequest { litres }), Err(_)) => {
                    let pints = litres * 1.7597539864
                    ;
                    let pints_request = PintsRequest { pints };
                    let response = serde_json::to_string(&pints_request).unwrap();
                    (StatusCode::OK, response)
                }
                (Err(_), Err(_), Err(_), Ok(PintsRequest { pints })) => {
                    let litres = pints * 0.56826125;
                    let litres_request = LitresRequest { litres };
                    let response = serde_json::to_string(&litres_request).unwrap();
                    (StatusCode::OK, response)
                }
                _ => (StatusCode::BAD_REQUEST, "".to_string()),
            }
        },
        Some(_) | None => {
            if milk_state.limiter.lock().unwrap().try_acquire(1) {
                (StatusCode::OK, "Milk withdrawn\n".to_string())
            } else {
                (StatusCode::TOO_MANY_REQUESTS, "No milk available\n".to_string())
            }
        }
    }

}

pub async fn refill_milk(
    State(milk_state): State<MilkState>,
) -> impl IntoResponse {
    *milk_state.limiter.lock().unwrap() = RateLimiter::builder()
        .max(MAX_MILK)
        .initial(MAX_MILK)
        .interval(Duration::from_secs(1))
        .build();
    StatusCode::OK
}