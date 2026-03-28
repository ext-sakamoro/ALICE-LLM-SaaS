// SPDX-License-Identifier: AGPL-3.0-or-later
// ALICE-LLM-SaaS core-engine: ビジネスロジック

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::sync::{Arc, Mutex};

// ---------------------------------------------------------------------------
// 統計情報
// ---------------------------------------------------------------------------

/// サービス全体のリクエスト統計
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Stats {
    pub generations: u64,
    pub tokenizations: u64,
    pub optimizations: u64,
    pub model_queries: u64,
}

/// スレッド安全な共有ステート
pub type AppState = Arc<Mutex<Stats>>;

/// 統計情報をJSON Valueとして返す
pub fn build_stats_response(stats: &Stats) -> Value {
    json!({
        "generations": stats.generations,
        "tokenizations": stats.tokenizations,
        "optimizations": stats.optimizations,
        "model_queries": stats.model_queries,
    })
}

// ---------------------------------------------------------------------------
// リクエスト/レスポンス型
// ---------------------------------------------------------------------------

#[derive(Debug, Deserialize)]
pub struct GenerateRequest {
    pub prompt: String,
    pub model: Option<String>,
    pub max_tokens: Option<u32>,
    pub temperature: Option<f32>,
}

#[derive(Debug, Deserialize)]
pub struct TokenizeRequest {
    pub text: String,
    pub model: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct OptimizeRequest {
    pub model_id: String,
    pub target: String,
    pub dataset_size: Option<u64>,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct HealthResponse {
    pub status: &'static str,
    pub service: &'static str,
    pub version: &'static str,
}

// ---------------------------------------------------------------------------
// デフォルト定数
// ---------------------------------------------------------------------------

/// デフォルトモデル名
pub const DEFAULT_MODEL: &str = "alice-7b";
/// デフォルト最大トークン数
pub const DEFAULT_MAX_TOKENS: u32 = 256;
/// プロンプト切り詰め長
pub const PROMPT_PREVIEW_LEN: usize = 50;
/// デフォルトデータセットサイズ
pub const DEFAULT_DATASET_SIZE: u64 = 10_000;
/// 推定最適化時間（秒）
pub const ESTIMATED_OPTIMIZATION_DURATION_S: u64 = 300;

// ---------------------------------------------------------------------------
// ヘルスチェック
// ---------------------------------------------------------------------------

/// ヘルスレスポンスを構築する
pub fn build_health_response(version: &'static str) -> HealthResponse {
    HealthResponse {
        status: "ok",
        service: "alice-llm-core",
        version,
    }
}

// ---------------------------------------------------------------------------
// テキスト生成
// ---------------------------------------------------------------------------

/// プロンプトのトークン数を空白分割でカウントする
pub fn count_prompt_tokens(prompt: &str) -> usize {
    prompt.split_whitespace().count()
}

/// 生成レスポンスのプレビュー文字列を構築する
pub fn build_generated_text(prompt: &str) -> String {
    let end = prompt.len().min(PROMPT_PREVIEW_LEN);
    format!("Generated response for: {}", &prompt[..end])
}

/// デフォルト温度
pub fn default_temperature() -> f32 {
    0.7
}

/// 生成レスポンス全体をJSON Valueとして構築する
pub fn build_generate_response(
    request_id: &str,
    model: &str,
    prompt: &str,
    max_tokens: u32,
    temperature: f32,
) -> Value {
    json!({
        "request_id": request_id,
        "model": model,
        "prompt_tokens": count_prompt_tokens(prompt),
        "completion_tokens": max_tokens,
        "temperature": temperature,
        "text": build_generated_text(prompt),
        "finish_reason": "stop",
    })
}

// ---------------------------------------------------------------------------
// トークナイズ
// ---------------------------------------------------------------------------

/// テキストを疑似トークンIDのベクタに変換する（空白分割、1000起点のID付与）
pub fn tokenize_text(text: &str) -> Vec<u32> {
    text.split_whitespace()
        .enumerate()
        .map(|(i, _)| i as u32 + 1000)
        .collect()
}

/// トークナイズレスポンスをJSON Valueとして構築する
pub fn build_tokenize_response(model: &str, text: &str) -> Value {
    let tokens = tokenize_text(text);
    let count = tokens.len();
    json!({
        "model": model,
        "token_count": count,
        "tokens": tokens,
    })
}

// ---------------------------------------------------------------------------
// モデル一覧
// ---------------------------------------------------------------------------

/// 利用可能モデルの定義
#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct ModelInfo {
    pub id: &'static str,
    pub model_type: &'static str,
}

/// 利用可能モデル一覧を返す
pub fn available_models() -> Vec<ModelInfo> {
    vec![
        ModelInfo {
            id: "alice-7b",
            model_type: "llm",
        },
        ModelInfo {
            id: "alice-gan-v2",
            model_type: "gan",
        },
        ModelInfo {
            id: "alice-automl-v1",
            model_type: "automl",
        },
    ]
}

/// モデル一覧レスポンスをJSON Valueとして構築する
pub fn build_models_response() -> Value {
    json!({
        "models": [
            { "id": "alice-7b", "type": "llm", "params": "7B", "context_length": 8192 },
            { "id": "alice-gan-v2", "type": "gan", "params": "2B", "resolution": "1024x1024" },
            { "id": "alice-automl-v1", "type": "automl", "tasks": ["classification", "regression"] },
        ]
    })
}

// ---------------------------------------------------------------------------
// 最適化ジョブ
// ---------------------------------------------------------------------------

/// 最適化レスポンスをJSON Valueとして構築する
pub fn build_optimize_response(
    job_id: &str,
    model_id: &str,
    target: &str,
    dataset_size: u64,
) -> Value {
    json!({
        "job_id": job_id,
        "model_id": model_id,
        "target": target,
        "dataset_size": dataset_size,
        "status": "queued",
        "estimated_duration_s": ESTIMATED_OPTIMIZATION_DURATION_S,
    })
}

// ---------------------------------------------------------------------------
// ユーティリティ: モデル名のデフォルト解決
// ---------------------------------------------------------------------------

/// Option<String>からモデル名を解決する
pub fn resolve_model(model: Option<String>) -> String {
    model.unwrap_or_else(|| DEFAULT_MODEL.to_string())
}

/// Option<u32>からmax_tokensを解決する
pub fn resolve_max_tokens(max_tokens: Option<u32>) -> u32 {
    max_tokens.unwrap_or(DEFAULT_MAX_TOKENS)
}

/// Option<f32>からtemperatureを解決する
pub fn resolve_temperature(temperature: Option<f32>) -> f32 {
    temperature.unwrap_or(default_temperature())
}

/// Option<u64>からdataset_sizeを解決する
pub fn resolve_dataset_size(dataset_size: Option<u64>) -> u64 {
    dataset_size.unwrap_or(DEFAULT_DATASET_SIZE)
}

// ===========================================================================
// テスト
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // -----------------------------------------------------------------------
    // Stats テスト
    // -----------------------------------------------------------------------

    #[test]
    fn stats_default_is_zero() {
        let s = Stats::default();
        assert_eq!(s.generations, 0);
        assert_eq!(s.tokenizations, 0);
        assert_eq!(s.optimizations, 0);
        assert_eq!(s.model_queries, 0);
    }

    #[test]
    fn stats_increment_generations() {
        let mut s = Stats::default();
        s.generations += 1;
        assert_eq!(s.generations, 1);
    }

    #[test]
    fn stats_increment_tokenizations() {
        let mut s = Stats::default();
        s.tokenizations += 5;
        assert_eq!(s.tokenizations, 5);
    }

    #[test]
    fn stats_increment_optimizations() {
        let mut s = Stats::default();
        s.optimizations += 3;
        assert_eq!(s.optimizations, 3);
    }

    #[test]
    fn stats_increment_model_queries() {
        let mut s = Stats::default();
        s.model_queries += 10;
        assert_eq!(s.model_queries, 10);
    }

    #[test]
    fn stats_accumulation_mixed() {
        // 複数フィールドを同時に加算
        let mut s = Stats::default();
        s.generations += 2;
        s.tokenizations += 3;
        s.optimizations += 1;
        s.model_queries += 4;
        assert_eq!(s.generations, 2);
        assert_eq!(s.tokenizations, 3);
        assert_eq!(s.optimizations, 1);
        assert_eq!(s.model_queries, 4);
    }

    #[test]
    fn stats_clone_eq() {
        let mut s = Stats::default();
        s.generations = 42;
        let s2 = s.clone();
        assert_eq!(s, s2);
    }

    #[test]
    fn build_stats_response_default() {
        let s = Stats::default();
        let v = build_stats_response(&s);
        assert_eq!(v["generations"], 0);
        assert_eq!(v["tokenizations"], 0);
        assert_eq!(v["optimizations"], 0);
        assert_eq!(v["model_queries"], 0);
    }

    #[test]
    fn build_stats_response_with_values() {
        let s = Stats {
            generations: 10,
            tokenizations: 20,
            optimizations: 5,
            model_queries: 30,
        };
        let v = build_stats_response(&s);
        assert_eq!(v["generations"], 10);
        assert_eq!(v["tokenizations"], 20);
        assert_eq!(v["optimizations"], 5);
        assert_eq!(v["model_queries"], 30);
    }

    #[test]
    fn app_state_shared_mutation() {
        // Arc<Mutex<Stats>>を介した共有変更
        let state: AppState = Arc::new(Mutex::new(Stats::default()));
        {
            let mut s = state.lock().unwrap();
            s.generations += 1;
        }
        let s = state.lock().unwrap();
        assert_eq!(s.generations, 1);
    }

    // -----------------------------------------------------------------------
    // ヘルスチェック テスト
    // -----------------------------------------------------------------------

    #[test]
    fn health_response_fields() {
        let h = build_health_response("0.1.0");
        assert_eq!(h.status, "ok");
        assert_eq!(h.service, "alice-llm-core");
        assert_eq!(h.version, "0.1.0");
    }

    #[test]
    fn health_response_custom_version() {
        let h = build_health_response("1.2.3");
        assert_eq!(h.version, "1.2.3");
    }

    // -----------------------------------------------------------------------
    // プロンプトトークンカウント テスト
    // -----------------------------------------------------------------------

    #[test]
    fn count_prompt_tokens_normal() {
        assert_eq!(count_prompt_tokens("hello world"), 2);
    }

    #[test]
    fn count_prompt_tokens_empty() {
        assert_eq!(count_prompt_tokens(""), 0);
    }

    #[test]
    fn count_prompt_tokens_whitespace_only() {
        assert_eq!(count_prompt_tokens("   "), 0);
    }

    #[test]
    fn count_prompt_tokens_single_word() {
        assert_eq!(count_prompt_tokens("hello"), 1);
    }

    #[test]
    fn count_prompt_tokens_multiple_spaces() {
        // 連続スペースは空白分割で無視される
        assert_eq!(count_prompt_tokens("a  b   c"), 3);
    }

    #[test]
    fn count_prompt_tokens_newlines_and_tabs() {
        assert_eq!(count_prompt_tokens("a\nb\tc"), 3);
    }

    // -----------------------------------------------------------------------
    // 生成テキスト構築 テスト
    // -----------------------------------------------------------------------

    #[test]
    fn build_generated_text_short_prompt() {
        let result = build_generated_text("hi");
        assert_eq!(result, "Generated response for: hi");
    }

    #[test]
    fn build_generated_text_long_prompt() {
        // 50文字を超えるプロンプトは切り詰められる
        let long = "a".repeat(100);
        let result = build_generated_text(&long);
        let expected = format!("Generated response for: {}", "a".repeat(PROMPT_PREVIEW_LEN));
        assert_eq!(result, expected);
    }

    #[test]
    fn build_generated_text_exact_50_chars() {
        let exact = "b".repeat(50);
        let result = build_generated_text(&exact);
        assert!(result.contains(&exact));
    }

    #[test]
    fn build_generated_text_empty() {
        let result = build_generated_text("");
        assert_eq!(result, "Generated response for: ");
    }

    // -----------------------------------------------------------------------
    // 生成レスポンス構築 テスト
    // -----------------------------------------------------------------------

    #[test]
    fn build_generate_response_basic() {
        let v = build_generate_response("req-1", "alice-7b", "hello world", 100, 0.5);
        assert_eq!(v["request_id"], "req-1");
        assert_eq!(v["model"], "alice-7b");
        assert_eq!(v["prompt_tokens"], 2);
        assert_eq!(v["completion_tokens"], 100);
        assert_eq!(v["finish_reason"], "stop");
    }

    #[test]
    fn build_generate_response_temperature() {
        let v = build_generate_response("r", "m", "x", 10, 1.0);
        // f64として比較
        let temp = v["temperature"].as_f64().unwrap();
        assert!((temp - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn build_generate_response_empty_prompt() {
        let v = build_generate_response("r", "m", "", 10, 0.5);
        assert_eq!(v["prompt_tokens"], 0);
        assert_eq!(v["text"], "Generated response for: ");
    }

    // -----------------------------------------------------------------------
    // トークナイズ テスト
    // -----------------------------------------------------------------------

    #[test]
    fn tokenize_text_normal() {
        let tokens = tokenize_text("hello world");
        assert_eq!(tokens, vec![1000, 1001]);
    }

    #[test]
    fn tokenize_text_empty() {
        let tokens = tokenize_text("");
        assert!(tokens.is_empty());
    }

    #[test]
    fn tokenize_text_whitespace_only() {
        let tokens = tokenize_text("   \t\n  ");
        assert!(tokens.is_empty());
    }

    #[test]
    fn tokenize_text_single_word() {
        let tokens = tokenize_text("alice");
        assert_eq!(tokens, vec![1000]);
    }

    #[test]
    fn tokenize_text_five_words() {
        let tokens = tokenize_text("a b c d e");
        assert_eq!(tokens, vec![1000, 1001, 1002, 1003, 1004]);
    }

    #[test]
    fn tokenize_text_ids_start_at_1000() {
        let tokens = tokenize_text("first");
        assert_eq!(tokens[0], 1000);
    }

    #[test]
    fn tokenize_text_ids_sequential() {
        let tokens = tokenize_text("one two three");
        for (i, &t) in tokens.iter().enumerate() {
            assert_eq!(t, i as u32 + 1000);
        }
    }

    #[test]
    fn build_tokenize_response_basic() {
        let v = build_tokenize_response("alice-7b", "hello world");
        assert_eq!(v["model"], "alice-7b");
        assert_eq!(v["token_count"], 2);
        let arr = v["tokens"].as_array().unwrap();
        assert_eq!(arr.len(), 2);
    }

    #[test]
    fn build_tokenize_response_empty_text() {
        let v = build_tokenize_response("m", "");
        assert_eq!(v["token_count"], 0);
        let arr = v["tokens"].as_array().unwrap();
        assert!(arr.is_empty());
    }

    // -----------------------------------------------------------------------
    // モデル一覧 テスト
    // -----------------------------------------------------------------------

    #[test]
    fn available_models_count() {
        assert_eq!(available_models().len(), 3);
    }

    #[test]
    fn available_models_contains_llm() {
        let models = available_models();
        assert!(models
            .iter()
            .any(|m| m.id == "alice-7b" && m.model_type == "llm"));
    }

    #[test]
    fn available_models_contains_gan() {
        let models = available_models();
        assert!(models
            .iter()
            .any(|m| m.id == "alice-gan-v2" && m.model_type == "gan"));
    }

    #[test]
    fn available_models_contains_automl() {
        let models = available_models();
        assert!(models
            .iter()
            .any(|m| m.id == "alice-automl-v1" && m.model_type == "automl"));
    }

    #[test]
    fn build_models_response_has_three_entries() {
        let v = build_models_response();
        let arr = v["models"].as_array().unwrap();
        assert_eq!(arr.len(), 3);
    }

    #[test]
    fn build_models_response_llm_context_length() {
        let v = build_models_response();
        let arr = v["models"].as_array().unwrap();
        let llm = arr.iter().find(|m| m["id"] == "alice-7b").unwrap();
        assert_eq!(llm["context_length"], 8192);
    }

    #[test]
    fn build_models_response_gan_resolution() {
        let v = build_models_response();
        let arr = v["models"].as_array().unwrap();
        let gan = arr.iter().find(|m| m["id"] == "alice-gan-v2").unwrap();
        assert_eq!(gan["resolution"], "1024x1024");
    }

    #[test]
    fn build_models_response_automl_tasks() {
        let v = build_models_response();
        let arr = v["models"].as_array().unwrap();
        let automl = arr.iter().find(|m| m["id"] == "alice-automl-v1").unwrap();
        let tasks = automl["tasks"].as_array().unwrap();
        assert_eq!(tasks.len(), 2);
        assert!(tasks.contains(&json!("classification")));
        assert!(tasks.contains(&json!("regression")));
    }

    // -----------------------------------------------------------------------
    // 最適化レスポンス テスト
    // -----------------------------------------------------------------------

    #[test]
    fn build_optimize_response_basic() {
        let v = build_optimize_response("job-1", "alice-7b", "latency", 5000);
        assert_eq!(v["job_id"], "job-1");
        assert_eq!(v["model_id"], "alice-7b");
        assert_eq!(v["target"], "latency");
        assert_eq!(v["dataset_size"], 5000);
        assert_eq!(v["status"], "queued");
        assert_eq!(v["estimated_duration_s"], ESTIMATED_OPTIMIZATION_DURATION_S);
    }

    #[test]
    fn build_optimize_response_default_dataset() {
        let v = build_optimize_response("j", "m", "t", DEFAULT_DATASET_SIZE);
        assert_eq!(v["dataset_size"], DEFAULT_DATASET_SIZE);
    }

    // -----------------------------------------------------------------------
    // デフォルト解決 テスト
    // -----------------------------------------------------------------------

    #[test]
    fn resolve_model_none_returns_default() {
        assert_eq!(resolve_model(None), DEFAULT_MODEL);
    }

    #[test]
    fn resolve_model_some_returns_value() {
        assert_eq!(resolve_model(Some("gpt-4".to_string())), "gpt-4");
    }

    #[test]
    fn resolve_max_tokens_none() {
        assert_eq!(resolve_max_tokens(None), DEFAULT_MAX_TOKENS);
    }

    #[test]
    fn resolve_max_tokens_some() {
        assert_eq!(resolve_max_tokens(Some(512)), 512);
    }

    #[test]
    fn resolve_temperature_none() {
        let t = resolve_temperature(None);
        assert!((t - 0.7).abs() < f32::EPSILON);
    }

    #[test]
    fn resolve_temperature_some() {
        let t = resolve_temperature(Some(1.5));
        assert!((t - 1.5).abs() < f32::EPSILON);
    }

    #[test]
    fn resolve_dataset_size_none() {
        assert_eq!(resolve_dataset_size(None), DEFAULT_DATASET_SIZE);
    }

    #[test]
    fn resolve_dataset_size_some() {
        assert_eq!(resolve_dataset_size(Some(99)), 99);
    }

    // -----------------------------------------------------------------------
    // 定数値 テスト
    // -----------------------------------------------------------------------

    #[test]
    fn default_model_is_alice_7b() {
        assert_eq!(DEFAULT_MODEL, "alice-7b");
    }

    #[test]
    fn default_max_tokens_is_256() {
        assert_eq!(DEFAULT_MAX_TOKENS, 256);
    }

    #[test]
    fn default_temperature_is_0_7() {
        assert!((default_temperature() - 0.7).abs() < f32::EPSILON);
    }

    #[test]
    fn prompt_preview_len_is_50() {
        assert_eq!(PROMPT_PREVIEW_LEN, 50);
    }

    #[test]
    fn default_dataset_size_is_10000() {
        assert_eq!(DEFAULT_DATASET_SIZE, 10_000);
    }

    #[test]
    fn estimated_optimization_duration_is_300() {
        assert_eq!(ESTIMATED_OPTIMIZATION_DURATION_S, 300);
    }
}
