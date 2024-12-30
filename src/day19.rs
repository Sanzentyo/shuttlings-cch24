use axum::{
    extract::{State, Path, Query},
    response::IntoResponse,
    http::StatusCode,
    Json,
};
use sqlx::{self, Row};
use std::sync::Arc;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use serde_json;
use rand::{distributions::Alphanumeric, thread_rng, Rng};
use std::collections::HashMap;
use tokio::sync::Mutex;
use once_cell::sync::Lazy;

// トークンを保存するためのグローバル状態
static TOKEN_STORE: Lazy<Mutex<HashMap<String, i32>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

// ランダムなトークンを生成する関数
fn generate_token() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect()
}

#[derive(Clone)]
pub struct StatePostgres {
    pub pool: Arc<sqlx::PgPool>,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow, Clone)]
pub struct Quote {
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

const INSERT_QUOTE_DRAFT_SQL: &str = "INSERT INTO quotes (id, author, quote, version) VALUES ($1, $2, $3, 1) RETURNING id;";

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
                println!("Undo quote: {:?}", quote);
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

    println!("Draft quote: {:?} with {}", quote, new_id);

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

#[derive(Debug, Serialize, Deserialize)]
pub struct Token {
    pub token: Option<[char; 16]>
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ListResponse {
    pub quotes: Vec<Quote>,
    pub page: i32,
    pub next_token: Option<String>
}

#[axum::debug_handler]
pub async fn list_db(
    State(pool): State<StatePostgres>,
    Query(params): Query<HashMap<String, String>>
) -> impl IntoResponse {
    let pool = &*pool.pool;
    let mut store = TOKEN_STORE.lock().await;

    println!("List quotes with params: {:?}", params);
    
    // トークンがある場合、対応するページを取得
    let current_page = if let Some(token) = params.get("token") {
        match store.get(token) {
            Some(&page) => {
                store.remove(token); // 使用済みトークンを削除
                page
            },
            None => return (StatusCode::BAD_REQUEST, "Invalid token".to_string())
        }
    } else {
        1
    };

    let offset = (current_page - 1) * 3;
    
    // クエリを実行してquotesを取得
    let quotes = match sqlx::query_as::<_, Quote>(
        "SELECT * FROM quotes ORDER BY created_at ASC LIMIT 4 OFFSET $1"
    )
    .bind(offset)
    .fetch_all(pool)
    .await {
        Ok(quotes) => quotes,
        Err(e) => return (StatusCode::NOT_FOUND, format!("Failed to fetch quotes: {}", e))
    };

    println!("List quotes: {:?}", quotes);

    // 次のページがあるか確認
    let (quotes, next_token) = if quotes.len() > 3 {
        let next_token = generate_token();
        store.insert(next_token.clone(), current_page + 1);
        (&quotes[..3], Some(next_token)) // 修正: Option<String> をそのまま使用
    } else {
        (&quotes[..], None)
    };

    let response = ListResponse {
        quotes: quotes.to_vec(),
        page: current_page,
        next_token
    };

    println!("List response: {:?}", response);

    (StatusCode::OK, serde_json::to_string(&response).unwrap())
}