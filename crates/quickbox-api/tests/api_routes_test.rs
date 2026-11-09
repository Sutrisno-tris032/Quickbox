use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use quickbox_api::{create_app, AppState};
use quickbox_smtp::ingest::NewMailNotification;
use serde_json::{json, Value};
use std::sync::Arc;
use tokio::sync::broadcast;
use tower::ServiceExt;

#[tokio::test]
async fn test_inbox_lock_unlock_flow() {
    let pool = quickbox_core::db::init_db("sqlite::memory:").await.unwrap();
    let temp_dir = tempfile::tempdir().unwrap();
    let (tx, _rx) = broadcast::channel::<NewMailNotification>(16);

    let state = AppState {
        pool,
        attachments_dir: temp_dir.path().to_path_buf(),
        broadcast_tx: Arc::new(tx),
        jwt_secret: "test-secret-key-at-least-32-chars-long".to_string(),
    };

    let app = create_app(state);

    // 1. GET status of "budi" -> 200, is_locked = false
    let req = Request::builder()
        .uri("/api/inboxes/budi/status")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let val: Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(val["is_locked"], false);

    // 2. POST lock "budi" with "pass123"
    let lock_body = serde_json::to_vec(&json!({ "password": "pass123" })).unwrap();
    let req = Request::builder()
        .method("POST")
        .uri("/api/inboxes/budi/lock")
        .header("Content-Type", "application/json")
        .body(Body::from(lock_body))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 3. GET messages without token -> expect 401 Unauthorized
    let req = Request::builder()
        .uri("/api/inboxes/budi/messages")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    // 4. POST unlock with wrong password -> expect 401
    let unlock_wrong = serde_json::to_vec(&json!({ "password": "wrong" })).unwrap();
    let req = Request::builder()
        .method("POST")
        .uri("/api/inboxes/budi/unlock")
        .header("Content-Type", "application/json")
        .body(Body::from(unlock_wrong))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    // 5. POST unlock with correct password -> expect 200 with token
    let unlock_correct = serde_json::to_vec(&json!({ "password": "pass123" })).unwrap();
    let req = Request::builder()
        .method("POST")
        .uri("/api/inboxes/budi/unlock")
        .header("Content-Type", "application/json")
        .body(Body::from(unlock_correct))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let val: Value = serde_json::from_slice(&body).unwrap();
    let token = val["token"].as_str().unwrap().to_string();
    assert!(!token.is_empty());

    // 6. GET messages with token -> expect 200 OK
    let req = Request::builder()
        .uri("/api/inboxes/budi/messages")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 7. DELETE lock with token -> expect 200 OK
    let req = Request::builder()
        .method("DELETE")
        .uri("/api/inboxes/budi/lock")
        .header("Authorization", format!("Bearer {}", token))
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 8. GET messages again without token -> expect 200 OK because inbox is public again!
    let req = Request::builder()
        .uri("/api/inboxes/budi/messages")
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

