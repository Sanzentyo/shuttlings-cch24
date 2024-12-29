use axum::{
    routing::get,
    Router,    
};
use std::sync::{Mutex, Arc};
use leaky_bucket::RateLimiter;
use std::time::Duration;
use rand::SeedableRng;

mod day1;
mod day2;
mod day5;
mod day9;
mod day12;
mod day16;

#[shuttle_runtime::main]
async fn main() -> shuttle_axum::ShuttleAxum {
    let milk_state = day9::MilkState {
        limiter: Arc::new(Mutex::new(RateLimiter::builder()
            .max(day9::MAX_MILK)
            .initial(day9::MAX_MILK)
            .interval(Duration::from_secs(1))
            .build())),
    };

    let board_state = day12::StateBoard {
        board: Arc::new(Mutex::new(day12::Board::new())),
        seed: Arc::new(Mutex::new(rand::rngs::StdRng::seed_from_u64(2024))),
    };
    

    let router = Router::new()
        .route("/", get(day1::hello_bird)) // day1 task 1
        .route("/-1/seek", get(day1::seek_and_found)) // day1 task 2
        .route("/2/dest", get(day2::from_key_calc)) // day2 task 1
        .route("/2/key", get(day2::from_to_calc)) // day2 task 2
        .route("/2/v6/dest", get(day2::from_key_calc_v6)) // day2 task 3
        .route("/2/v6/key", get(day2::from_to_calc_v6)) // day2 task 3
        .route("/5/manifest", get(day5::return_manifest).post(day5::return_manifest)) // day5 task 1
        .route("/9/milk", get(day9::milk_and_cookies).post(day9::milk_and_cookies))// day9 task 1
        .route("/9/refill", get(day9::refill_milk).post(day9::refill_milk)) // day9 task 2
        .with_state(milk_state)
        .route("/12/reset", get(day12::reset).post(day12::reset)) // day12 task 1
        .route("/12/place/:team/:column", get(day12::place).post(day12::place)) // day12 task 2
        .route("/12/board", get(day12::get_board).post(day12::get_board)) // day12 task 3
        .route("/12/random-board", get(day12::rand_board).post(day12::rand_board)) // day12 task 4
        .with_state(board_state)
        .route("/16/wrap", get(day16::wrap).post(day16::wrap)); // day16 task 1

    Ok(router.into())
}