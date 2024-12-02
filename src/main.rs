use axum::{routing::get, Router, response::{Redirect, IntoResponse}, http::StatusCode};

async fn hello_bird() -> &'static str {
    "Hello, bird!"
}

async fn youtube_location() -> impl IntoResponse {
    (StatusCode::FOUND, Redirect::to("https://www.youtube.com/watch?v=9Gc4QTqslN4"))
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new()
        .route("/", get(hello_bird))
        .route("/-1/seek", get(youtube_location));

    Ok(router.into())
}
