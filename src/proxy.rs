use axum::{
    extract::State,
    http::{HeaderMap, Method, StatusCode},
    response::Response,
    routing::{get, post},
    Json, Router,
};
use serde_json::Value;
use tower_http::cors::{Any, CorsLayer};

use crate::util;

#[derive(Clone)]
pub struct AppState {
    openai_endpoint: String,
    ollama_endpoint: String,
    openai_api_key: Option<String>,
}

pub fn create_app(
    openai_endpoint: &str,
    ollama_endpoint: &str,
    openai_api_key: Option<&str>,
) -> Router {
    let state = AppState {
        openai_endpoint: openai_endpoint.to_string(),
        ollama_endpoint: ollama_endpoint.to_string(),
        openai_api_key: openai_api_key.map(String::from),
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    Router::new()
        .route("/*path", get(handle_get))
        .route("/*path", post(handle_post))
        .layer(cors)
        .with_state(state)
}

async fn handle_post(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<Value>,
) -> Result<Response<String>, StatusCode> {
    let model = payload["model"].as_str().ok_or(StatusCode::BAD_REQUEST)?;

    let endpoint = util::choose_endpoint(model, &state.ollama_endpoint, &state.openai_endpoint);

    // 驗證 API key
    if let Some(api_key) = &state.openai_api_key {
        let auth_header = headers
            .get("Authorization")
            .and_then(|h| h.to_str().ok())
            .unwrap_or("");

        if !auth_header.starts_with("Bearer ") || !auth_header.ends_with(api_key) {
            return Err(StatusCode::UNAUTHORIZED);
        }
    }

    // 轉發請求
    let client = reqwest::Client::new();
    let resp = client
        .post(&endpoint)
        .headers(headers)
        .json(&payload)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    Ok(Response::new(
        resp.text()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    ))
}

async fn handle_get(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response<String>, StatusCode> {
    let client = reqwest::Client::new();
    let resp = client
        .get(&state.ollama_endpoint)
        .headers(headers)
        .send()
        .await
        .map_err(|_| StatusCode::BAD_GATEWAY)?;

    Ok(Response::new(
        resp.text()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?,
    ))
}
