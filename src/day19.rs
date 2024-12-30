use axum::{
    extract::{State, Path},
    response::IntoResponse,
    http::StatusCode,
    Json,
};
use sqlx::{self, Row};
use std::sync::Arc;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use serde_json;

#[derive(Clone)]
pub struct StatePostgres {
    pub pool: Arc<sqlx::PgPool>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
struct Quote {
    id: Uuid,
    author: String,
    quote: String,
    created_at: chrono::DateTime<chrono::Utc>,
    version: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DraftQuote {
    pub author: String,
    pub quote: String,
}

pub const MAKE_DB_SQL: &str = "CREATE TABLE IF NOT EXISTS quotes (
    id UUID PRIMARY KEY,
    author TEXT NOT NULL,
    quote TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT CURRENT_TIMESTAMP,
    version INT NOT NULL DEFAULT 1
);";

const RESET_DB_SQL: &str = "TRUNCATE quotes;";

const SELECT_QUOTE_SQL: &str = "SELECT * FROM quotes WHERE id = $1;";

const DELETE_QUOTE_SQL: &str = "DELETE FROM quotes WHERE id = $1;";

const UPDATE_QUOTE_DRAFT_SQL: &str = "UPDATE quotes SET author = $1, quote = $2, version = version + 1 WHERE id = $3;";

const INSERT_QUOTE_DRAFT_SQL: &str = "INSERT INTO quotes (id, author, quote) VALUES ($1, $2, $3) RETURNING id;";

pub async fn reset_db(
    State(pool): State<StatePostgres>
) -> impl IntoResponse {
    let pool = &*pool.pool;
    match sqlx::query(RESET_DB_SQL).execute(pool).await {
        Ok(pg_query_result) => {
            let rows_affected = pg_query_result.rows_affected();
            (StatusCode::OK, format!("Database reset successfully. Rows affected: {}", rows_affected))
        }
        Err(e) => (StatusCode::BAD_REQUEST, format!("Database reset failed: {}", e)),
    }
}

#[derive(Deserialize)]
pub struct DBParams {
    pub id: Uuid
}

pub async fn cite(
    State(pool): State<StatePostgres>,
    Path(params): Path<DBParams>
) -> impl IntoResponse {
    let id = params.id;
    let pool = &*pool.pool;

    let quote = match sqlx::query_as::<_, Quote>(SELECT_QUOTE_SQL)
        .bind(id)
        .fetch_one(pool)
        .await {
            Ok(quote) => quote,
            Err(e) => return (StatusCode::NOT_FOUND, format!("Failed to fetch quote: {}", e)),
        };

    (StatusCode::OK, serde_json::to_string(&quote).unwrap())
}

pub async fn remove_db(
    State(pool): State<StatePostgres>,
    Path(params): Path<DBParams>
) -> impl IntoResponse {
    let id = params.id;
    let pool = &*pool.pool;

    let delete_quote = match sqlx::query_as::<_, Quote>(SELECT_QUOTE_SQL)
        .bind(id)
        .fetch_one(pool)
        .await {
            Ok(quote) => quote,
            Err(e) => return (StatusCode::NOT_FOUND, format!("Failed to fetch quote: {}", e)),
        };

    match sqlx::query(DELETE_QUOTE_SQL)
        .bind(id)
        .execute(pool)
        .await {
            Ok(_) => {
                (StatusCode::OK, serde_json::to_string(&delete_quote).unwrap())
            }
            Err(e) => (StatusCode::NOT_FOUND, format!("Failed to remove quote: {}", e)),
        }
}

pub async fn undo_db(
    State(pool): State<StatePostgres>,
    Path(params): Path<DBParams>,
    Json(quote): Json<DraftQuote>
) -> impl IntoResponse {
    let pool = &*pool.pool;

    let id = params.id;

    match sqlx::query(UPDATE_QUOTE_DRAFT_SQL)
        .bind(quote.author)
        .bind(quote.quote)
        .bind(id)
        .execute(pool)
        .await {
            Ok(_) => {
                let quote = match sqlx::query_as::<_, Quote>(SELECT_QUOTE_SQL)
                    .bind(id)
                    .fetch_one(pool)
                    .await {
                        Ok(quote) => quote,
                        Err(e) => return (StatusCode::NOT_FOUND, format!("Failed to fetch quote: {}", e)),
                    };
                (StatusCode::OK, serde_json::to_string(&quote).unwrap())
            }
            Err(e) => {
                (StatusCode::NOT_FOUND, format!("Failed to update quote: {}", e))
            },
        }
}

pub async fn draft_db(
    State(pool): State<StatePostgres>,
    Json(quote): Json<DraftQuote>
) -> impl IntoResponse {
    let pool = &*pool.pool;

    let new_id = Uuid::new_v4(); // 新しいUUIDを生成

    let result = sqlx::query(INSERT_QUOTE_DRAFT_SQL)
        .bind(new_id) // 生成したUUIDをバインド
        .bind(&quote.author)
        .bind(&quote.quote)
        .fetch_one(pool)
        .await;

    match result {
        Ok(record) => {
            let inserted_id: Uuid = record.get("id");
            match sqlx::query_as::<_, Quote>(SELECT_QUOTE_SQL)
                .bind(inserted_id)
                .fetch_one(pool)
                .await {
                    Ok(quote) => (StatusCode::CREATED, serde_json::to_string(&quote).unwrap()),
                    Err(e) => (StatusCode::NOT_FOUND, format!("Failed to fetch quote: {}", e)),
                }
        }
        Err(e) => (StatusCode::NOT_FOUND, format!("Failed to insert quote: {}", e)),
    }
}