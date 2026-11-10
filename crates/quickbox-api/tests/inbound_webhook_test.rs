use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use quickbox_api::{create_app, AppState};
use quickbox_smtp::ingest::NewMailNotification;
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::broadcast;
use tower::ServiceExt;

const RAW_EMAIL: &[u8] = b"From: sender@example.com\r\n\
To: testing@quickbox.dev\r\n\
Subject: Test Inbound Webhook\r\n\
Content-Type: text/plain\r\n\
\r\n\
Kode OTP kamu: 482913\r\n\
.\r\n";

async fn build_state(inbound_secret: Option<&str>) -> AppState {
    let pool = quickbox_core::db::init_db("sqlite::memory:").await.unwrap();
    let temp_dir = tempfile::tempdir().unwrap();
    let (tx, _rx) = broadcast::channel::<NewMailNotification>(16);

    AppState {
        pool,
        attachments_dir: temp_dir.path().to_path_buf(),
        broadcast_tx: Arc::new(tx),
        jwt_secret: "test-secret-key-at-least-32-chars-long".to_string(),
        domain: "quickbox.dev".to_string(),
        inbound_secret: inbound_secret.map(|s| s.to_string()),
    }
}

#[tokio::test]
async fn test_inbound_webhook_returns_404_when_secret_not_configured() {
    let state = build_state(None).await;
    let app = create_app(state);

    let req = Request::builder()
        .method("POST")
        .uri("/internal/inbound")
        .header("X-Quickbox-Secret", "whatever")
        .header("X-Quickbox-Rcpt", "testing@quickbox.dev")
        .body(Body::from(RAW_EMAIL))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn test_inbound_webhook_rejects_wrong_secret() {
    let state = build_state(Some("correct-secret")).await;
    let app = create_app(state);

    let req = Request::builder()
        .method("POST")
        .uri("/internal/inbound")
        .header("X-Quickbox-Secret", "wrong-secret")
        .header("X-Quickbox-Rcpt", "testing@quickbox.dev")
        .body(Body::from(RAW_EMAIL))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_inbound_webhook_rejects_missing_secret_header() {
    let state = build_state(Some("correct-secret")).await;
    let app = create_app(state);

    let req = Request::builder()
        .method("POST")
        .uri("/internal/inbound")
        .header("X-Quickbox-Rcpt", "testing@quickbox.dev")
        .body(Body::from(RAW_EMAIL))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);
}

#[tokio::test]
async fn test_inbound_webhook_rejects_oversized_body() {
    let state = build_state(Some("correct-secret")).await;
    let app = create_app(state);

    let oversized = vec![0u8; 10 * 1024 * 1024 + 1];
    let req = Request::builder()
        .method("POST")
        .uri("/internal/inbound")
        .header("X-Quickbox-Secret", "correct-secret")
        .header("X-Quickbox-Rcpt", "testing@quickbox.dev")
        .body(Body::from(oversized))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::PAYLOAD_TOO_LARGE);
}

#[tokio::test]
async fn test_inbound_webhook_rejects_foreign_recipient_domain() {
    let state = build_state(Some("correct-secret")).await;
    let app = create_app(state);

    let req = Request::builder()
        .method("POST")
        .uri("/internal/inbound")
        .header("X-Quickbox-Secret", "correct-secret")
        .header("X-Quickbox-Rcpt", "testing@gmail.com")
        .body(Body::from(RAW_EMAIL))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_inbound_webhook_rejects_missing_recipient_header() {
    let state = build_state(Some("correct-secret")).await;
    let app = create_app(state);

    let req = Request::builder()
        .method("POST")
        .uri("/internal/inbound")
        .header("X-Quickbox-Secret", "correct-secret")
        .body(Body::from(RAW_EMAIL))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNPROCESSABLE_ENTITY);
}

#[tokio::test]
async fn test_inbound_webhook_accepts_and_ingests_valid_email() {
    let state = build_state(Some("correct-secret")).await;
    let pool = state.pool.clone();
    let app = create_app(state);

    let req = Request::builder()
        .method("POST")
        .uri("/internal/inbound")
        .header("X-Quickbox-Secret", "correct-secret")
        .header("X-Quickbox-Rcpt", "testing@quickbox.dev")
        .body(Body::from(RAW_EMAIL))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::ACCEPTED);

    let count: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM messages WHERE inbox_name = 'testing'")
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(count.0, 1);

    let req = Request::builder()
        .uri("/api/inboxes/testing/messages")
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let messages: Vec<Value> = serde_json::from_slice(&body).unwrap();
    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0]["detected_otp"], "482913");
}
