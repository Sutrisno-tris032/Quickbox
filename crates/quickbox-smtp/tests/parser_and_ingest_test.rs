use quickbox_smtp::extractor::{extract_links, extract_otp};
use quickbox_smtp::ingest::{ingest_email, NewMailNotification};
use std::sync::Arc;
use tokio::sync::broadcast;

#[tokio::test]
async fn test_otp_and_link_extraction() {
    let text1 = "Your verification code is 482910. Please do not share it.";
    assert_eq!(extract_otp(text1), Some("482910".to_string()));

    let text2 = "Gunakan kode OTP 938210 untuk verifikasi akun Anda. Tautan: https://app.test/verify?token=abc";
    assert_eq!(extract_otp(text2), Some("938210".to_string()));

    let links = extract_links(text2);
    assert_eq!(links, vec!["https://app.test/verify?token=abc"]);

    let text_none = "Halo apa kabar? Ini email biasa tanpa kode.";
    assert_eq!(extract_otp(text_none), None);
}

#[tokio::test]
async fn test_email_parsing_and_ingestion() {
    let pool = quickbox_core::db::init_db("sqlite::memory:").await.unwrap();
    let temp_dir = tempfile::tempdir().unwrap();
    let (tx, mut rx) = broadcast::channel::<NewMailNotification>(16);

    let raw_email = b"From: sender@example.com\r\n\
To: budi-test@quickbox.dev\r\n\
Subject: Test Verification Code\r\n\
Content-Type: text/plain; charset=utf-8\r\n\
\r\n\
Halo Budi, kode verifikasi Anda adalah 582913.\r\n";

    let recipients = vec!["budi-test@quickbox.dev".to_string()];
    let messages = ingest_email(
        raw_email,
        &recipients,
        &pool,
        temp_dir.path(),
        Arc::new(tx),
    )
    .await
    .unwrap();

    assert_eq!(messages.len(), 1);
    let msg = &messages[0];
    assert_eq!(msg.inbox_name, "budi-test");
    assert_eq!(msg.sender, "sender@example.com");
    assert_eq!(msg.subject, "Test Verification Code");
    assert_eq!(msg.detected_otp, Some("582913".to_string()));

    // Verify broadcast event was received
    let event = rx.recv().await.unwrap();
    assert_eq!(event.inbox_name, "budi-test");
    assert_eq!(event.message_id, msg.id);
}

#[tokio::test]
async fn test_inline_cid_image_is_rewritten_to_attachment_url() {
    let pool = quickbox_core::db::init_db("sqlite::memory:").await.unwrap();
    let temp_dir = tempfile::tempdir().unwrap();
    let (tx, _rx) = broadcast::channel::<NewMailNotification>(16);

    let raw_email = b"From: sender@example.com\r\n\
To: budi-test@quickbox.dev\r\n\
Subject: Inline Image Test\r\n\
Content-Type: multipart/related; boundary=\"BOUNDARY\"\r\n\
\r\n\
--BOUNDARY\r\n\
Content-Type: text/html; charset=utf-8\r\n\
\r\n\
<html><body><p>Lihat gambar:</p><img src=\"cid:logo123\"></body></html>\r\n\
--BOUNDARY\r\n\
Content-Type: image/png\r\n\
Content-Transfer-Encoding: base64\r\n\
Content-ID: <logo123>\r\n\
Content-Disposition: inline; filename=\"logo.png\"\r\n\
\r\n\
iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR42mNk+A8AAQUBAScY42YAAAAASUVORK5CYII=\r\n\
--BOUNDARY--\r\n";

    let recipients = vec!["budi-test@quickbox.dev".to_string()];
    let messages = ingest_email(raw_email, &recipients, &pool, temp_dir.path(), Arc::new(tx))
        .await
        .unwrap();

    assert_eq!(messages.len(), 1);
    let msg = &messages[0];
    let body_html = msg.body_html.clone().expect("expected html body");

    // The raw cid: reference must no longer be present...
    assert!(!body_html.contains("cid:logo123"));

    // ...and must be replaced with a URL to the actual stored attachment.
    let attachment: (String, String) =
        sqlx::query_as("SELECT id, content_type FROM attachments WHERE message_id = ?")
            .bind(&msg.id)
            .fetch_one(&pool)
            .await
            .unwrap();
    let expected_url = format!("/api/inboxes/budi-test/attachments/{}", attachment.0);
    assert!(body_html.contains(&expected_url));

    // The full MIME type (not just the top-level "image") must be stored, otherwise
    // browsers refuse to render the <img> since the Content-Type header is incomplete.
    assert_eq!(attachment.1, "image/png");
}

