use anyhow::Result;
use sqlx::SqlitePool;
use std::path::Path;
use tokio::fs;
use tracing::{debug, info};

#[derive(Debug, Default)]
pub struct CleanupSummary {
    pub expired_messages_deleted: u64,
    pub attachments_deleted: u64,
    pub stale_locks_released: u64,
}

pub async fn run_cleanup_cycle(
    pool: &SqlitePool,
    attachments_dir: &Path,
    retention_days: i64,
) -> Result<CleanupSummary> {
    let mut summary = CleanupSummary::default();
    let modifier = format!("-{} days", retention_days);

    // 1. Find and delete physical attachment files for expired messages
    let attachments_to_delete: Vec<(String, String)> = sqlx::query_as(
        r#"
        SELECT id, disk_path FROM attachments
        WHERE message_id IN (SELECT id FROM messages WHERE received_at < datetime('now', ?))
        "#,
    )
    .bind(&modifier)
    .fetch_all(pool)
    .await?;

    for (_id, disk_path) in attachments_to_delete {
        let full_path = attachments_dir.join(&disk_path);
        if full_path.exists() {
            if let Err(e) = fs::remove_file(&full_path).await {
                debug!("Failed to remove attachment file {}: {}", full_path.display(), e);
            } else {
                summary.attachments_deleted += 1;
            }
        }
        // Remove empty parent directory if possible
        if let Some(parent) = full_path.parent() {
            let _ = fs::remove_dir(parent).await;
        }
    }

    // 2. Delete expired messages (CASCADE will delete DB attachments)
    let res = sqlx::query("DELETE FROM messages WHERE received_at < datetime('now', ?)")
        .bind(&modifier)
        .execute(pool)
        .await?;
    summary.expired_messages_deleted = res.rows_affected();

    // 3. Release stale locks where user has been inactive for > retention_days
    let res = sqlx::query(
        r#"
        UPDATE inboxes
        SET is_locked = 0, password_hash = NULL, locked_at = NULL
        WHERE is_locked = 1 AND last_user_activity_at < datetime('now', ?)
        "#,
    )
    .bind(&modifier)
    .execute(pool)
    .await?;
    summary.stale_locks_released = res.rows_affected();

    info!(
        "Retention cycle complete: {} messages, {} attachments, {} stale locks released",
        summary.expired_messages_deleted,
        summary.attachments_deleted,
        summary.stale_locks_released
    );

    Ok(summary)
}
