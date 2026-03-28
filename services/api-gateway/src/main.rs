// SPDX-License-Identifier: AGPL-3.0-or-later
// ALICE-LLM-SaaS api-gateway

use alice_llm_gateway::{
    build_gateway_config, build_health_response, build_listen_addr, build_proxy_error_response,
    build_upstream_health_url, increment_request_count, new_gateway_state, total_request_count,
    AppState,
};
use axum::{extract::State, http::StatusCode, response::Json, routing::get, Router};
use serde_json::Value;
use std::sync::Arc;
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;

/// ヘルスチェックハンドラ
async fn health(State(state): State<AppState>) -> Json<Value> {
    let total = total_request_count(&state);
    Json(build_health_response(&state.upstream, total))
}

/// アップストリームへのプロキシハンドラ
async fn proxy_handler(State(state): State<AppState>) -> (StatusCode, Json<Value>) {
    increment_request_count(&state, "proxied");
    let url = build_upstream_health_url(&state.upstream);
    match state.client.get(&url).send().await {
        Ok(resp) => {
            let body: Value = resp.json().await.unwrap_or(serde_json::json!({}));
            (StatusCode::OK, Json(body))
        }
        Err(e) => (
            StatusCode::BAD_GATEWAY,
            Json(build_proxy_error_response(&e.to_string())),
        ),
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let config = build_gateway_config(
        std::env::var("UPSTREAM_URL").ok().as_deref(),
        std::env::var("GATEWAY_PORT").ok().as_deref(),
    );

    let state: AppState = Arc::new(new_gateway_state(config.upstream_url));

    let app = Router::new()
        .route("/health", get(health))
        .route("/upstream/health", get(proxy_handler))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let addr = build_listen_addr(config.port);
    info!("alice-llm-gateway listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
