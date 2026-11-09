use quickbox_core::models::Message;
use quickbox_core::retention::run_cleanup_cycle;

#[tokio::test]
async fn test_retention_and_auto_release_inactive_lock() {
    let pool = quickbox_core::db::init_db("sqlite::memory:").await.unwrap();
    let temp_dir = tempfile::tempdir().unwrap();

    // 1. Create stale locked inbox (inactive 35 days ago)
    sqlx::query(
        r#"
        INSERT INTO inboxes (name, is_locked, password_hash, created_at, last_user_activity_at)
        VALUES ('stale-inbox', 1, 'hash123', datetime('now', '-40 days'), datetime('now', '-35 days'))
        "#
    )
    .execute(&pool)
    .await
    .unwrap();

    // 2. Create active locked inbox (active 5 days ago)
    sqlx::query(
        r#"
        INSERT INTO inboxes (name, is_locked, password_hash, created_at, last_user_activity_at)
        VALUES ('active-inbox', 1, 'hash456', datetime('now', '-10 days'), datetime('now', '-5 days'))
        "#
    )
    .execute(&pool)
    .await
    .unwrap();

    // 3. Create expired message (received 35 days ago) with attachment
    sqlx::query(
        r#"
        INSERT INTO messages (id, inbox_name, sender, recipient, subject, snippet, raw_headers, received_at)
        VALUES ('msg-old', 'stale-inbox', 's@test.com', 'stale-inbox@quickbox.dev', 'Old', '', '', datetime('now', '-35 days'))
        "#
    )
    .execute(&pool)
    .await
    .unwrap();

    let att_path = temp_dir.path().join("msg-old").join("test.pdf");
    tokio::fs::create_dir_all(att_path.parent().unwrap()).await.unwrap();
    tokio::fs::write(&att_path, b"dummy pdf content").await.unwrap();

    sqlx::query(
        r#"
        INSERT INTO attachments (id, message_id, filename, disk_path, size_bytes)
        VALUES ('att-1', 'msg-old', 'test.pdf', 'msg-old/test.pdf', 17)
        "#
    )
    .execute(&pool)
    .await
    .unwrap();

    // 4. Create recent message (received 2 days ago)
    sqlx::query(
        r#"
        INSERT INTO messages (id, inbox_name, sender, recipient, subject, snippet, raw_headers, received_at)
        VALUES ('msg-new', 'active-inbox', 's@test.com', 'active-inbox@quickbox.dev', 'New', '', '', datetime('now', '-2 days'))
        "#
    )
    .execute(&pool)
    .await
    .unwrap();

    // 5. Run cleanup cycle with 30-day retention
    let summary = run_cleanup_cycle(&pool, temp_dir.path(), 30).await.unwrap();

    assert_eq!(summary.expired_messages_deleted, 1);
    assert_eq!(summary.attachments_deleted, 1);
    assert_eq!(summary.stale_locks_released, 1);

    // Verify disk file deleted
    assert!(!att_path.exists(), "Attachment file should be deleted from disk");

    // Verify messages in DB
    let remaining_msgs: Vec<Message> = sqlx::query_as("SELECT * FROM messages ORDER BY id")
        .fetch_all(&pool)
        .await
        .unwrap();
    assert_eq!(remaining_msgs.len(), 1);
    assert_eq!(remaining_msgs[0].id, "msg-new");

    // Verify inbox lock states
    let stale_inbox = quickbox_core::db::get_or_create_inbox(&pool, "stale-inbox").await.unwrap();
    assert_eq!(stale_inbox.is_locked, false, "Stale inbox should be unlocked automatically");
    assert!(stale_inbox.password_hash.is_none());

    let active_inbox = quickbox_core::db::get_or_create_inbox(&pool, "active-inbox").await.unwrap();
    assert_eq!(active_inbox.is_locked, true, "Active locked inbox should remain locked");
}
