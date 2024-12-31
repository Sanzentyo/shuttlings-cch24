use axum::{
    extract::Path,
    response::{Html, IntoResponse},
    http::StatusCode,
    extract::Multipart,
};
use serde::Deserialize;
use html_escape::encode_text;
use toml::Value;
use std::collections::HashSet;

pub async fn star_lit() -> impl IntoResponse {
    Html("<div id=\"star\" class=\"lit\"></div>")
}

#[derive(Deserialize)]
pub struct ColorParams {
    color: String,
}

fn make_present_div(color: &str) -> String {
    format!(
        "<div class=\"present {}\" hx-get=\"/23/present/{}\" hx-swap=\"outerHTML\">
            <div class=\"ribbon\"></div>
            <div class=\"ribbon\"></div>
            <div class=\"ribbon\"></div>
            <div class=\"ribbon\"></div>
        </div>",
        color,
        match color {
            "red" => "blue",
            "blue" => "purple",
            "purple" => "red",
            _ => ""
        }
    )
}

pub async fn present_color(
    Path(ColorParams{color}): Path<ColorParams>
) -> impl IntoResponse {
    match color.as_str() {
        "red" | "blue" | "purple" => {
            (StatusCode::OK, Html(make_present_div(&color)))
        },
        _ => (StatusCode::IM_A_TEAPOT, Html("".to_string()))
    }
}

#[derive(Deserialize)]
pub struct OrnamentParams {
    state: String,
    n: String,
}

pub async fn ornament(
    Path(OrnamentParams { state, n }): Path<OrnamentParams>
) -> impl IntoResponse {
    println!("Ornament {} is {}", n, state);

    let n = encode_text(&n).replace('"', "&quot;");
    let state = encode_text(&state).replace('"', "&quot;");


    match state.as_str() {
        "on" => {
            (StatusCode::OK, Html(format!(
                r#"<div class="ornament on" id="ornament{}" hx-trigger="load delay:2s once" hx-get="/23/ornament/off/{}" hx-swap="outerHTML"></div>"#,
                n, n
            )))
        },
        "off" => {
            (StatusCode::OK, Html(format!(
                r#"<div class="ornament" id="ornament{}" hx-trigger="load delay:2s once" hx-get="/23/ornament/on/{}" hx-swap="outerHTML"></div>"#,
                n, n
            )))
        },
        _ => (StatusCode::IM_A_TEAPOT, Html("".to_string()))
    }
}

pub async fn lockfile(mut multipart: Multipart) -> impl IntoResponse {
    let field = match multipart.next_field().await {
        Ok(Some(field)) if field.name() == Some("lockfile") => field,
        _ => return (StatusCode::BAD_REQUEST, Html("".to_string()))
    };

    let content = match field.bytes().await {
        Ok(bytes) => bytes,
        _ => return (StatusCode::BAD_REQUEST, Html("".to_string()))
    };

    let toml_content = match std::str::from_utf8(&content) {
        Ok(s) => s,
        _ => return (StatusCode::BAD_REQUEST, Html("".to_string()))
    };

    let value: Value = match toml::from_str(toml_content) {
        Ok(v) => v,
        _ => return (StatusCode::BAD_REQUEST, Html("".to_string()))
    };

    let mut html = String::new();
    let mut seen = HashSet::new();

    if let Value::Table(table) = value {
        if let Some(Value::Array(packages)) = table.get("package") {
            for package in packages {
                if let Value::Table(pkg) = package {
                    match pkg.get("checksum") {
                        Some(Value::String(checksum)) => {
                            if seen.contains(checksum) {
                                continue;
                            }
                            seen.insert(checksum.clone());

                            // 最低10文字必要
                            if checksum.len() < 10 {
                                return (StatusCode::UNPROCESSABLE_ENTITY, Html("".to_string()));
                            }

                            // 16進数文字列の検証
                            if !checksum[..10].chars().all(|c| c.is_ascii_hexdigit()) {
                                return (StatusCode::UNPROCESSABLE_ENTITY, Html("".to_string()));
                            }

                            let color = &checksum[..6];
                            let top = match u8::from_str_radix(&checksum[6..8], 16) {
                                Ok(v) => v,
                                Err(_) => return (StatusCode::UNPROCESSABLE_ENTITY, Html("".to_string())),
                            };
                            let left = match u8::from_str_radix(&checksum[8..10], 16) {
                                Ok(v) => v,
                                Err(_) => return (StatusCode::UNPROCESSABLE_ENTITY, Html("".to_string())),
                            };

                            html.push_str(&format!(
                                "<div style=\"background-color:#{};top:{}px;left:{}px;\"></div>\n",
                                color, top, left
                            ));
                        },
                        Some(Value::Array(_)) | Some(_) | None => {
                            continue;
                        }
                    }
                }
            }
        }
    }

    if html.is_empty() {
        return (StatusCode::BAD_REQUEST, Html("".to_string()));
    }

    (StatusCode::OK, Html(html))
}