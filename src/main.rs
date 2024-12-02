use axum::{routing::get, Router};

async fn hello_bird() -> &'static str {
    "Hello, bird!"
}

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new().route("/", get(hello_bird));

    Ok(router.into())
}
