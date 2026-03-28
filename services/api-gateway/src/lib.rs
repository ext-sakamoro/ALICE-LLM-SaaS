// SPDX-License-Identifier: AGPL-3.0-or-later
// ALICE-LLM-SaaS api-gateway ライブラリ

use dashmap::DashMap;
use serde_json::{json, Value};
use std::sync::Arc;

// ---------------------------------------------------------------------------
// ゲートウェイ状態
// ---------------------------------------------------------------------------

/// ゲートウェイの共有状態
pub struct GatewayState {
    /// アップストリームURL
    pub upstream: String,
    /// HTTPクライアント
    pub client: reqwest::Client,
    /// エンドポイント毎のリクエストカウント
    pub request_counts: DashMap<String, u64>,
}

/// アプリケーション状態の型エイリアス
pub type AppState = Arc<GatewayState>;

// ---------------------------------------------------------------------------
// リクエストカウント操作
// ---------------------------------------------------------------------------

/// 指定キーのカウンタをインクリメントし、新しい値を返す
pub fn increment_request_count(state: &GatewayState, key: &str) -> u64 {
    let mut entry = state.request_counts.entry(key.to_string()).or_insert(0);
    *entry += 1;
    *entry
}

/// 全キーの合計リクエスト数を取得
pub fn total_request_count(state: &GatewayState) -> u64 {
    state.request_counts.iter().map(|e| *e.value()).sum()
}

/// 指定キーのリクエスト数を取得（存在しなければ0）
pub fn get_request_count(state: &GatewayState, key: &str) -> u64 {
    state
        .request_counts
        .get(key)
        .map(|e| *e.value())
        .unwrap_or(0)
}

// ---------------------------------------------------------------------------
// ヘルスレスポンス構築
// ---------------------------------------------------------------------------

/// ヘルスチェック用JSONレスポンスを構築
pub fn build_health_response(upstream: &str, total_requests: u64) -> Value {
    json!({
        "status": "ok",
        "service": "alice-llm-gateway",
        "upstream": upstream,
        "total_requests": total_requests,
    })
}

// ---------------------------------------------------------------------------
// ゲートウェイ設定
// ---------------------------------------------------------------------------

/// ゲートウェイの起動設定
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GatewayConfig {
    /// アップストリームURL
    pub upstream_url: String,
    /// リッスンポート
    pub port: u16,
}

/// デフォルトのアップストリームURL
pub const DEFAULT_UPSTREAM_URL: &str = "http://localhost:8136";

/// デフォルトのゲートウェイポート
pub const DEFAULT_GATEWAY_PORT: u16 = 9136;

/// 環境変数からアップストリームURLを取得（未設定時はデフォルト）
pub fn resolve_upstream_url(env_val: Option<&str>) -> String {
    match env_val {
        Some(v) if !v.is_empty() => v.to_string(),
        _ => DEFAULT_UPSTREAM_URL.to_string(),
    }
}

/// 環境変数からポートを取得（未設定・パース失敗時はデフォルト）
pub fn resolve_gateway_port(env_val: Option<&str>) -> u16 {
    match env_val {
        Some(v) => v.parse::<u16>().unwrap_or(DEFAULT_GATEWAY_PORT),
        None => DEFAULT_GATEWAY_PORT,
    }
}

/// 環境変数のペアからゲートウェイ設定を構築
pub fn build_gateway_config(upstream_env: Option<&str>, port_env: Option<&str>) -> GatewayConfig {
    GatewayConfig {
        upstream_url: resolve_upstream_url(upstream_env),
        port: resolve_gateway_port(port_env),
    }
}

/// リッスンアドレス文字列を構築
pub fn build_listen_addr(port: u16) -> String {
    format!("0.0.0.0:{port}")
}

/// プロキシ先のヘルスエンドポイントURLを構築
pub fn build_upstream_health_url(upstream: &str) -> String {
    format!("{upstream}/health")
}

// ---------------------------------------------------------------------------
// プロキシレスポンスユーティリティ
// ---------------------------------------------------------------------------

/// アップストリームエラー時のレスポンスボディを構築
pub fn build_proxy_error_response(error_msg: &str) -> Value {
    json!({ "error": error_msg })
}

/// `GatewayState` を新規作成するヘルパー
pub fn new_gateway_state(upstream: String) -> GatewayState {
    GatewayState {
        upstream,
        client: reqwest::Client::new(),
        request_counts: DashMap::new(),
    }
}

// ---------------------------------------------------------------------------
// テスト
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;

    // ===== ヘルパー =====

    /// テスト用のデフォルト状態を作成
    fn test_state() -> GatewayState {
        new_gateway_state("http://test-upstream:8000".to_string())
    }

    // ===== リクエストカウント =====

    #[test]
    fn カウント_初回インクリメントで1になる() {
        let state = test_state();
        let count = increment_request_count(&state, "test");
        assert_eq!(count, 1);
    }

    #[test]
    fn カウント_連続インクリメントで累積する() {
        let state = test_state();
        increment_request_count(&state, "key");
        increment_request_count(&state, "key");
        let count = increment_request_count(&state, "key");
        assert_eq!(count, 3);
    }

    #[test]
    fn カウント_異なるキーは独立() {
        let state = test_state();
        increment_request_count(&state, "a");
        increment_request_count(&state, "a");
        increment_request_count(&state, "b");
        assert_eq!(get_request_count(&state, "a"), 2);
        assert_eq!(get_request_count(&state, "b"), 1);
    }

    #[test]
    fn カウント_存在しないキーは0() {
        let state = test_state();
        assert_eq!(get_request_count(&state, "missing"), 0);
    }

    #[test]
    fn カウント_合計が正しい() {
        let state = test_state();
        increment_request_count(&state, "x");
        increment_request_count(&state, "y");
        increment_request_count(&state, "y");
        assert_eq!(total_request_count(&state), 3);
    }

    #[test]
    fn カウント_空の状態で合計0() {
        let state = test_state();
        assert_eq!(total_request_count(&state), 0);
    }

    #[test]
    fn カウント_多数キーの合計() {
        let state = test_state();
        for i in 0..100 {
            increment_request_count(&state, &format!("key_{i}"));
        }
        assert_eq!(total_request_count(&state), 100);
    }

    #[test]
    fn カウント_同一キー大量インクリメント() {
        let state = test_state();
        for _ in 0..1000 {
            increment_request_count(&state, "hot");
        }
        assert_eq!(get_request_count(&state, "hot"), 1000);
    }

    #[test]
    fn カウント_並行インクリメント() {
        let state = Arc::new(test_state());
        let mut handles = vec![];
        for _ in 0..10 {
            let s = Arc::clone(&state);
            handles.push(std::thread::spawn(move || {
                for _ in 0..100 {
                    increment_request_count(&s, "concurrent");
                }
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        assert_eq!(get_request_count(&state, "concurrent"), 1000);
    }

    #[test]
    fn カウント_並行異なるキー() {
        let state = Arc::new(test_state());
        let mut handles = vec![];
        for i in 0..10 {
            let s = Arc::clone(&state);
            handles.push(std::thread::spawn(move || {
                for _ in 0..50 {
                    increment_request_count(&s, &format!("thread_{i}"));
                }
            }));
        }
        for h in handles {
            h.join().unwrap();
        }
        assert_eq!(total_request_count(&state), 500);
    }

    #[test]
    fn カウント_空文字キーも有効() {
        let state = test_state();
        increment_request_count(&state, "");
        assert_eq!(get_request_count(&state, ""), 1);
    }

    #[test]
    fn カウント_日本語キー() {
        let state = test_state();
        increment_request_count(&state, "リクエスト");
        assert_eq!(get_request_count(&state, "リクエスト"), 1);
    }

    // ===== ヘルスレスポンス =====

    #[test]
    fn ヘルス_ステータスがok() {
        let resp = build_health_response("http://up", 0);
        assert_eq!(resp["status"], "ok");
    }

    #[test]
    fn ヘルス_サービス名が正しい() {
        let resp = build_health_response("http://up", 0);
        assert_eq!(resp["service"], "alice-llm-gateway");
    }

    #[test]
    fn ヘルス_アップストリームが反映される() {
        let resp = build_health_response("http://my-upstream:9999", 0);
        assert_eq!(resp["upstream"], "http://my-upstream:9999");
    }

    #[test]
    fn ヘルス_リクエスト数が反映される() {
        let resp = build_health_response("http://up", 42);
        assert_eq!(resp["total_requests"], 42);
    }

    #[test]
    fn ヘルス_リクエスト数0() {
        let resp = build_health_response("http://up", 0);
        assert_eq!(resp["total_requests"], 0);
    }

    #[test]
    fn ヘルス_大きなリクエスト数() {
        let resp = build_health_response("http://up", u64::MAX);
        assert_eq!(resp["total_requests"], u64::MAX);
    }

    #[test]
    fn ヘルス_フィールド数が4() {
        let resp = build_health_response("http://up", 0);
        let obj = resp.as_object().unwrap();
        assert_eq!(obj.len(), 4);
    }

    // ===== ゲートウェイ設定 =====

    #[test]
    fn 設定_デフォルトのアップストリーム_url() {
        let url = resolve_upstream_url(None);
        assert_eq!(url, DEFAULT_UPSTREAM_URL);
    }

    #[test]
    fn 設定_カスタムアップストリーム_url() {
        let url = resolve_upstream_url(Some("http://custom:1234"));
        assert_eq!(url, "http://custom:1234");
    }

    #[test]
    fn 設定_空文字のアップストリームはデフォルト() {
        let url = resolve_upstream_url(Some(""));
        assert_eq!(url, DEFAULT_UPSTREAM_URL);
    }

    #[test]
    fn 設定_デフォルトのポート() {
        let port = resolve_gateway_port(None);
        assert_eq!(port, DEFAULT_GATEWAY_PORT);
    }

    #[test]
    fn 設定_カスタムポート() {
        let port = resolve_gateway_port(Some("3000"));
        assert_eq!(port, 3000);
    }

    #[test]
    fn 設定_不正なポート文字列はデフォルト() {
        let port = resolve_gateway_port(Some("abc"));
        assert_eq!(port, DEFAULT_GATEWAY_PORT);
    }

    #[test]
    fn 設定_ポート範囲外はデフォルト() {
        // u16::MAX = 65535 を超える値
        let port = resolve_gateway_port(Some("99999"));
        assert_eq!(port, DEFAULT_GATEWAY_PORT);
    }

    #[test]
    fn 設定_ポート0は有効() {
        let port = resolve_gateway_port(Some("0"));
        assert_eq!(port, 0);
    }

    #[test]
    fn 設定_build_gateway_config全デフォルト() {
        let cfg = build_gateway_config(None, None);
        assert_eq!(
            cfg,
            GatewayConfig {
                upstream_url: DEFAULT_UPSTREAM_URL.to_string(),
                port: DEFAULT_GATEWAY_PORT,
            }
        );
    }

    #[test]
    fn 設定_build_gateway_config全カスタム() {
        let cfg = build_gateway_config(Some("http://x:1"), Some("4000"));
        assert_eq!(cfg.upstream_url, "http://x:1");
        assert_eq!(cfg.port, 4000);
    }

    // ===== ユーティリティ =====

    #[test]
    fn ユーティリティ_リッスンアドレス構築() {
        assert_eq!(build_listen_addr(8080), "0.0.0.0:8080");
    }

    #[test]
    fn ユーティリティ_アップストリームヘルス_url() {
        let url = build_upstream_health_url("http://localhost:8136");
        assert_eq!(url, "http://localhost:8136/health");
    }

    #[test]
    fn ユーティリティ_プロキシエラーレスポンス() {
        let resp = build_proxy_error_response("connection refused");
        assert_eq!(resp["error"], "connection refused");
    }

    #[test]
    fn ユーティリティ_プロキシエラーはフィールド1つ() {
        let resp = build_proxy_error_response("timeout");
        let obj = resp.as_object().unwrap();
        assert_eq!(obj.len(), 1);
    }

    #[test]
    fn ユーティリティ_new_gateway_stateのアップストリーム() {
        let state = new_gateway_state("http://foo".to_string());
        assert_eq!(state.upstream, "http://foo");
    }

    #[test]
    fn ユーティリティ_new_gateway_stateの初期カウント空() {
        let state = new_gateway_state("http://foo".to_string());
        assert!(state.request_counts.is_empty());
    }
}
