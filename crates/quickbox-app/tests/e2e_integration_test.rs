use axum::body::Body;
use axum::http::{Request, StatusCode};
use http_body_util::BodyExt;
use quickbox_api::{create_app, AppState};
use quickbox_smtp::ingest::NewMailNotification;
use quickbox_smtp::SmtpServer;
use serde_json::Value;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::broadcast;
use tower::ServiceExt;

#[tokio::test]
async fn test_full_e2e_monolith_flow() {
    let pool = quickbox_core::db::init_db("sqlite::memory:").await.unwrap();
    let temp_dir = tempfile::tempdir().unwrap();
    let (tx, _rx) = broadcast::channel::<NewMailNotification>(32);
    let broadcast_tx = Arc::new(tx);

    // 1. Start SMTP listener
    let smtp_server = SmtpServer::bind("127.0.0.1:0", "quickbox.dev", 10_000_000)
        .await
        .unwrap()
        .with_storage(
            pool.clone(),
            temp_dir.path().to_path_buf(),
            broadcast_tx.clone(),
        );
    let smtp_addr = smtp_server.local_addr().unwrap();
    tokio::spawn(smtp_server.serve());

    // 2. Setup Axum app
    let state = AppState {
        pool: pool.clone(),
        attachments_dir: temp_dir.path().to_path_buf(),
        broadcast_tx: broadcast_tx.clone(),
        jwt_secret: "secret-key-for-e2e-testing-purposes".to_string(),
        domain: "quickbox.dev".to_string(),
        inbound_secret: None,
    };
    let app = create_app(state);

    // 3. Connect via SMTP TCP socket and send an email
    let stream = TcpStream::connect(smtp_addr).await.unwrap();
    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    reader.read_line(&mut line).await.unwrap(); // 220 banner

    writer.write_all(b"EHLO client.test\r\n").await.unwrap();
    line.clear();
    // Read multiline 250 response
    loop {
        line.clear();
        reader.read_line(&mut line).await.unwrap();
        if line.len() >= 4 && &line[3..4] == " " {
            break;
        }
    }

    writer.write_all(b"MAIL FROM:<auth@app.dev>\r\n").await.unwrap();
    line.clear();
    reader.read_line(&mut line).await.unwrap();
    assert!(line.starts_with("250"));

    writer.write_all(b"RCPT TO:<budi-e2e@quickbox.dev>\r\n").await.unwrap();
    line.clear();
    reader.read_line(&mut line).await.unwrap();
    assert!(line.starts_with("250"));

    writer.write_all(b"DATA\r\n").await.unwrap();
    line.clear();
    reader.read_line(&mut line).await.unwrap();
    assert!(line.starts_with("354"));

    let email_body = b"From: auth@app.dev\r\n\
To: budi-e2e@quickbox.dev\r\n\
Subject: Welcome to DevApp\r\n\
Content-Type: text/plain; charset=utf-8\r\n\
\r\n\
Gunakan kode OTP 839102 untuk verifikasi akun Anda.\r\n\
.\r\n";

    writer.write_all(email_body).await.unwrap();
    line.clear();
    reader.read_line(&mut line).await.unwrap();
    assert!(line.starts_with("250"));

    writer.write_all(b"QUIT\r\n").await.unwrap();

    // 4. Query Axum HTTP API
    let req = Request::builder()
        .uri("/api/inboxes/budi-e2e/messages")
        .body(Body::empty())
        .unwrap();

    let resp = app.oneshot(req).await.unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body_bytes = resp.into_body().collect().await.unwrap().to_bytes();
    let messages: Vec<Value> = serde_json::from_slice(&body_bytes).unwrap();

    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0]["subject"], "Welcome to DevApp");
    assert_eq!(messages[0]["sender"], "auth@app.dev");
    assert_eq!(messages[0]["detected_otp"], "839102");
}

