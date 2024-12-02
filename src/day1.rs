use axum::{http::StatusCode, response::{IntoResponse, Redirect}};

// task 1
pub async fn hello_bird() -> &'static str {
    "Hello, bird!"
}

// task 2
pub async fn seek_and_found() -> impl IntoResponse {
    (StatusCode::FOUND, Redirect::to("https://www.youtube.com/watch?v=9Gc4QTqslN4"))
}