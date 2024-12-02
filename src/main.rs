use axum::{
    routing::get,
    Router,    
};

mod day1;
mod day2;




#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let router = Router::new()
        .route("/", get(day1::hello_bird)) // day1 task 1
        .route("/-1/seek", get(day1::seek_and_found)) // day1 task 2
        .route("/2/dest", get(day2::from_key_calc)) // day2 task 1
        .route("/2/key", get(day2::from_to_calc)) // day2 task 2
        .route("/2/v6/dest", get(day2::from_key_calc_v6)) // day2 task 3
        .route("/2/v6/key", get(day2::from_to_calc_v6)); // day2 task 3

    Ok(router.into())
}
