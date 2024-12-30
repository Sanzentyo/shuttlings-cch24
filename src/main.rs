use axum::{
    routing::get,
    Router,    
};
use std::sync::{Mutex, Arc};
use leaky_bucket::RateLimiter;
use std::time::Duration;
use rand::SeedableRng;
//use shuttle_shared_db;
//use sqlx;

mod day1;
mod day2;
mod day5;
mod day9;
mod day12;
mod day16;
//mod day19;

#[shuttle_runtime::main]
async fn main(/*#[shuttle_shared_db::Postgres] pool: sqlx::PgPool*/) -> shuttle_axum::ShuttleAxum {
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

    /*sqlx::query(day19::MAKE_DB_SQL)
        .execute(&pool)
        .await
        .unwrap();
    let state_pool = day19::StatePostgres {
        pool: Arc::new(pool),
    };*/
    

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
        .route("/16/wrap", get(day16::wrap).post(day16::wrap)) // day16 task 1
        .route("/16/unwrap", get(day16::unwrap).post(day16::unwrap)) // day16 task 2
        .route("/16/decode", get(day16::decode_santa).post(day16::decode_santa)) // day16 task 2
        /*.route("/19/reset", get(day19::reset_db).post(day19::reset_db)) // day19 task 1
        .route("/19/cite/:id", get(day19::cite).post(day19::cite)) // day19 task 2
        .route("/19/remove/:id", get(day19::remove_db).delete(day19::remove_db)) // day19 task 3
        .route("/19/undo/:id", get(day19::undo_db).put(day19::undo_db).post(day19::undo_db)) // day19 task 3
        .route("/19/draft", get(day19::draft_db).post(day19::draft_db)) // day19 task 4
        .route("/19/list", get(day19::list_db).post(day19::list_db)) // day19 task 5
        .with_state(state_pool)*/
        ;

    Ok(router.into())
}