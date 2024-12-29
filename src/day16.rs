use axum::{
    extract::{State, Path},
    http::StatusCode,
    response::IntoResponse,
};

pub async fn wrap() -> impl IntoResponse {
    StatusCode::OK
}