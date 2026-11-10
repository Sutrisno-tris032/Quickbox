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
        domain: "quickbox.dev".to_string(),
        inbound_secret: None,
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
        .body(Body::from(lock_body.clone()))
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

    // 9. SECURITY REGRESSION: Cannot lock an already locked inbox (anti-takeover)
    // Lock it first
    let req = Request::builder()
        .method("POST")
        .uri("/api/inboxes/budi/lock")
        .header("Content-Type", "application/json")
        .body(Body::from(lock_body.clone()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Attempt second lock -> expect 409 Conflict
    let req = Request::builder()
        .method("POST")
        .uri("/api/inboxes/budi/lock")
        .header("Content-Type", "application/json")
        .body(Body::from(lock_body))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::CONFLICT);

    // 10. AUTH REGRESSION: Inboxes with dots (e.g. "budi.santoso") work with token auth
    let dot_inbox = "budi.santoso";
    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/inboxes/{}/lock", dot_inbox))
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&json!({ "password": "pwd" })).unwrap()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let req = Request::builder()
        .method("POST")
        .uri(format!("/api/inboxes/{}/unlock", dot_inbox))
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&json!({ "password": "pwd" })).unwrap()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let val: Value = serde_json::from_slice(&body).unwrap();
    let dot_token = val["token"].as_str().unwrap();

    let req = Request::builder()
        .uri(format!("/api/inboxes/{}/messages", dot_inbox))
        .header("Authorization", format!("Bearer {}", dot_token))
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn test_attachment_idor_and_authenticated_download() {
    let pool = quickbox_core::db::init_db("sqlite::memory:").await.unwrap();
    let temp_dir = tempfile::tempdir().unwrap();
    let (tx, _rx) = broadcast::channel::<NewMailNotification>(16);

    let state = AppState {
        pool: pool.clone(),
        attachments_dir: temp_dir.path().to_path_buf(),
        broadcast_tx: Arc::new(tx),
        jwt_secret: "test-secret-key-at-least-32-chars-long".to_string(),
        domain: "quickbox.dev".to_string(),
        inbound_secret: None,
    };

    let app = create_app(state);

    // Create an inbox for "alice" and an inbox for "bob"
    let _ = quickbox_core::db::get_or_create_inbox(&pool, "alice").await.unwrap();
    let _ = quickbox_core::db::get_or_create_inbox(&pool, "bob").await.unwrap();

    // Insert a message for alice
    let msg_id = "msg-123";
    sqlx::query(
        "INSERT INTO messages (id, inbox_name, sender, recipient, subject, snippet, body_text, body_html, raw_headers, is_read, is_deleted) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 0, 0)"
    )
    .bind(msg_id)
    .bind("alice")
    .bind("sender@example.com")
    .bind("alice@quickbox.dev")
    .bind("Subject")
    .bind("Snippet")
    .bind("Body")
    .bind("<p>Body</p>")
    .bind("headers")
    .execute(&pool)
    .await
    .unwrap();

    // Create a dummy attachment file on disk
    let att_id = "att-456";
    let rel_path = "msg-123/test.txt";
    let file_path = temp_dir.path().join(rel_path);
    tokio::fs::create_dir_all(file_path.parent().unwrap()).await.unwrap();
    tokio::fs::write(&file_path, b"secret attachment content").await.unwrap();

    // Insert attachment in DB
    sqlx::query(
        "INSERT INTO attachments (id, message_id, filename, content_type, size_bytes, disk_path, created_at) VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)"
    )
    .bind(att_id)
    .bind(msg_id)
    .bind("test.txt")
    .bind("text/plain")
    .bind(25)
    .bind(rel_path)
    .execute(&pool)
    .await
    .unwrap();

    // 1. GET attachment via alice -> 200 OK
    let req = Request::builder()
        .uri(format!("/api/inboxes/alice/attachments/{}", att_id))
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 2. IDOR check: GET attachment via bob -> 404 NOT FOUND
    let req = Request::builder()
        .uri(format!("/api/inboxes/bob/attachments/{}", att_id))
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);

    // 3. IDOR check: list attachments for msg_id via bob -> empty list
    let req = Request::builder()
        .uri(format!("/api/inboxes/bob/messages/{}/attachments", msg_id))
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let atts: Vec<quickbox_core::models::Attachment> = serde_json::from_slice(&body).unwrap();
    assert!(atts.is_empty(), "Bob should not see Alice's message attachments");

    // 4. Lock alice inbox (this purges previous public messages)
    let lock_body = serde_json::to_vec(&json!({ "password": "alice-secret" })).unwrap();
    let req = Request::builder()
        .method("POST")
        .uri("/api/inboxes/alice/lock")
        .header("Content-Type", "application/json")
        .body(Body::from(lock_body.clone()))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // Re-insert message and attachment now that alice is locked
    sqlx::query(
        "INSERT INTO messages (id, inbox_name, sender, recipient, subject, snippet, body_text, body_html, raw_headers, is_read, is_deleted) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, 0, 0)"
    )
    .bind(msg_id)
    .bind("alice")
    .bind("sender@example.com")
    .bind("alice@quickbox.dev")
    .bind("Subject")
    .bind("Snippet")
    .bind("Body")
    .bind("<p>Body</p>")
    .bind("headers")
    .execute(&pool)
    .await
    .unwrap();

    sqlx::query(
        "INSERT INTO attachments (id, message_id, filename, content_type, size_bytes, disk_path, created_at) VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)"
    )
    .bind(att_id)
    .bind(msg_id)
    .bind("test.txt")
    .bind("text/plain")
    .bind(25)
    .bind(rel_path)
    .execute(&pool)
    .await
    .unwrap();

    // 5. GET attachment without token -> 401 UNAUTHORIZED
    let req = Request::builder()
        .uri(format!("/api/inboxes/alice/attachments/{}", att_id))
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::UNAUTHORIZED);

    // 6. Unlock alice to obtain token
    let req = Request::builder()
        .method("POST")
        .uri("/api/inboxes/alice/unlock")
        .header("Content-Type", "application/json")
        .body(Body::from(lock_body))
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
    let body = resp.into_body().collect().await.unwrap().to_bytes();
    let val: Value = serde_json::from_slice(&body).unwrap();
    let token = val["token"].as_str().unwrap();

    // 7. GET attachment with ?token=... query parameter -> 200 OK
    let req = Request::builder()
        .uri(format!("/api/inboxes/alice/attachments/{}?token={}", att_id, token))
        .body(Body::empty())
        .unwrap();
    let resp = app.clone().oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

