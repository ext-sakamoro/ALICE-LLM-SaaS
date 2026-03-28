// SPDX-License-Identifier: AGPL-3.0-or-later
// ALICE-LLM-SaaS core-engine: ルーティングとサーバー起動のみ

use alice_llm_core::{
    build_generate_response, build_health_response, build_models_response, build_optimize_response,
    build_stats_response, build_tokenize_response, resolve_dataset_size, resolve_max_tokens,
    resolve_model, resolve_temperature, AppState, GenerateRequest, HealthResponse, OptimizeRequest,
    Stats, TokenizeRequest,
};
use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde_json::Value;
use std::sync::{Arc, Mutex};
use tower_http::{cors::CorsLayer, trace::TraceLayer};
use tracing::info;
use uuid::Uuid;

/// ヘルスチェックハンドラ
async fn health() -> Json<HealthResponse> {
    Json(build_health_response(env!("CARGO_PKG_VERSION")))
}

/// テキスト生成ハンドラ
async fn generate(
    State(state): State<AppState>,
    Json(req): Json<GenerateRequest>,
) -> (StatusCode, Json<Value>) {
    let request_id = Uuid::new_v4().to_string();
    let model = resolve_model(req.model);
    let max_tokens = resolve_max_tokens(req.max_tokens);
    let temperature = resolve_temperature(req.temperature);
    {
        let mut s = state.lock().unwrap();
        s.generations += 1;
    }
    info!(request_id = %request_id, model = %model, "LLM generation request");
    let response =
        build_generate_response(&request_id, &model, &req.prompt, max_tokens, temperature);
    (StatusCode::OK, Json(response))
}

/// トークナイズハンドラ
async fn tokenize(
    State(state): State<AppState>,
    Json(req): Json<TokenizeRequest>,
) -> (StatusCode, Json<Value>) {
    let model = resolve_model(req.model);
    {
        let mut s = state.lock().unwrap();
        s.tokenizations += 1;
    }
    let response = build_tokenize_response(&model, &req.text);
    (StatusCode::OK, Json(response))
}

/// モデル一覧ハンドラ
async fn list_models(State(state): State<AppState>) -> Json<Value> {
    {
        let mut s = state.lock().unwrap();
        s.model_queries += 1;
    }
    Json(build_models_response())
}

/// 最適化ジョブ作成ハンドラ
async fn optimize(
    State(state): State<AppState>,
    Json(req): Json<OptimizeRequest>,
) -> (StatusCode, Json<Value>) {
    let job_id = Uuid::new_v4().to_string();
    let dataset_size = resolve_dataset_size(req.dataset_size);
    {
        let mut s = state.lock().unwrap();
        s.optimizations += 1;
    }
    info!(job_id = %job_id, model_id = %req.model_id, target = %req.target, "AutoML optimization started");
    let response = build_optimize_response(&job_id, &req.model_id, &req.target, dataset_size);
    (StatusCode::ACCEPTED, Json(response))
}

/// 統計情報ハンドラ
async fn stats(State(state): State<AppState>) -> Json<Value> {
    let s = state.lock().unwrap();
    Json(build_stats_response(&s))
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let state: AppState = Arc::new(Mutex::new(Stats::default()));

    let app = Router::new()
        .route("/health", get(health))
        .route("/api/v1/llm/generate", post(generate))
        .route("/api/v1/llm/tokenize", post(tokenize))
        .route("/api/v1/llm/models", get(list_models))
        .route("/api/v1/llm/optimize", post(optimize))
        .route("/api/v1/llm/stats", get(stats))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let port = std::env::var("PORT").unwrap_or_else(|_| "8136".to_string());
    let addr = format!("0.0.0.0:{port}");
    info!("alice-llm-core listening on {}", addr);
    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
