use axum::{
    body::Bytes, extract::Json, http::{header::SET_COOKIE, HeaderMap, StatusCode}, response::IntoResponse
};
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation, Algorithm};
use jsonwebtoken::errors::ErrorKind;
use serde::{Serialize, Deserialize};
use serde_json::Value;
use chrono::{Duration, Utc};
use std::collections::HashSet;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    data: Value,
    exp: usize,
}

const JWT_SECRET: &[u8] = b"santa_secret_key";

pub async fn wrap(Json(body): Json<Value>) -> impl IntoResponse {
    let exp = (Utc::now() + Duration::days(1)).timestamp() as usize;
    let claims = Claims {
        data: body,
        exp,
    };
    
    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(JWT_SECRET)
    ).unwrap();

    let cookie = format!("gift={}", token);
    
    (
        StatusCode::OK,
        [(SET_COOKIE, cookie)]
    )
}

pub async fn unwrap(
    cookie: Option<HeaderMap>,
) -> impl IntoResponse {
    let cookie = if let Some(cookie) = cookie{
        cookie
    } else {
        return (StatusCode::BAD_REQUEST, "".to_string());
    };

    let token = if let Some(token) = cookie.get("cookie") {
        token
    } else {
        return (StatusCode::BAD_REQUEST, "".to_string());
    };
    let token = if let Ok(token) = token.to_str() {
        token
    } else {
        return (StatusCode::BAD_REQUEST, "".to_string());
    };

    let token = token.replace("gift=", "");
    
    let claims = if let Ok(token_data) = decode::<Value>(
        &token,
        &DecodingKey::from_secret(JWT_SECRET),
        &Validation::default()
    ) {
        token_data.claims
    } else {
        return (StatusCode::BAD_REQUEST, "".to_string());
    };

    (
        StatusCode::OK,
        claims.get("data").unwrap().to_string()
    )
}

const SANTA_JWT_SECRET: &[u8] = include_bytes!("day16_santa_public_key.pem");
use std::sync::LazyLock;

static SANTA_ENCODING_KEY: LazyLock<DecodingKey> = LazyLock::new(|| {
    DecodingKey::from_rsa_pem(SANTA_JWT_SECRET).unwrap()
});

pub async fn decode_santa(
    body_bytes: Bytes
) -> impl IntoResponse {
    let body = body_bytes.to_vec();
    let body = if let Ok(body) = String::from_utf8(body) {
        body
    } else {
        return (StatusCode::BAD_REQUEST, "".to_string());
    };

    let mut validation = Validation::new(Algorithm::RS256);
    validation.validate_exp = false;
    validation.required_spec_claims = HashSet::new();

    let algorithms = vec![
        Algorithm::HS256,
        Algorithm::HS384,
        Algorithm::HS512,
        Algorithm::RS256,
        Algorithm::RS384,
        Algorithm::RS512,
        Algorithm::ES256,
        Algorithm::ES384,
        Algorithm::PS256,
        Algorithm::PS256,
        Algorithm::PS384,
        Algorithm::PS512,
    ];

    let mut is_bad_request = false;
    for alg in algorithms {
        validation.algorithms = vec![alg];
        match decode::<Value>(
            &body,
            &SANTA_ENCODING_KEY,
            &validation
        ) {
            Ok(token_data) => {
                //println!("{:?}", token_data.claims);
                return (
                    StatusCode::OK,
                    token_data.claims.to_string()
                );
            },
            Err(e) => {
                //println!("Failed to decode with {:?}: {:?}", alg, e);
                match e.kind() {
                    ErrorKind::InvalidToken | ErrorKind::Json(_) | ErrorKind::Base64(_) => {
                        is_bad_request = true;
                    },
                    _ => {}
                }
            }
        }
    }
    if is_bad_request {
        (
            StatusCode::BAD_REQUEST,
            "Failed to decode with all algorithms".to_string()
        )
    } else {
        (
            StatusCode::UNAUTHORIZED,
            "Failed to decode".to_string()
        )
    }
}