use crate::extractor::{extract_links, extract_otp};
use crate::parser::ParsedEmail;
use anyhow::Result;
use quickbox_core::models::Message;
use serde::{Deserialize, Serialize};
use sqlx::SqlitePool;
use std::path::Path;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::broadcast;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewMailNotification {
    pub inbox_name: String,
    pub message_id: String,
    pub subject: String,
    pub sender: String,
}

pub async fn ingest_email(
    raw_bytes: &[u8],
    recipients: &[String],
    pool: &SqlitePool,
    attachments_dir: &Path,
    broadcast_tx: Arc<broadcast::Sender<NewMailNotification>>,
) -> Result<Vec<Message>> {
    let parsed = ParsedEmail::parse(raw_bytes)?;

    // Extract OTP & links
    let full_text = format!(
        "{}\n{}\n{}",
        parsed.subject,
        parsed.body_text.as_deref().unwrap_or(""),
        parsed.body_html.as_deref().unwrap_or("")
    );
    let detected_otp = extract_otp(&full_text);
    let detected_links = {
        let links = extract_links(&full_text);
        if links.is_empty() {
            None
        } else {
            Some(serde_json::to_string(&links)?)
        }
    };

    let mut saved_messages = Vec::new();

    for recipient in recipients {
        let local_part = recipient
            .split_once('@')
            .map(|(l, _)| l)
            .unwrap_or(recipient)
            .trim()
            .to_lowercase();

        // Ensure inbox exists and update last_received_at
        quickbox_core::db::get_or_create_inbox(pool, &local_part).await?;
        sqlx::query("UPDATE inboxes SET last_received_at = CURRENT_TIMESTAMP WHERE name = ?")
            .bind(&local_part)
            .execute(pool)
            .await?;

        let message_id = Uuid::new_v4().to_string();

        sqlx::query(
            r#"
            INSERT INTO messages (
                id, inbox_name, sender, recipient, subject, snippet,
                body_text, body_html, raw_headers, detected_otp, detected_links,
                is_read, is_favorite, is_deleted, received_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, 0, 0, 0, CURRENT_TIMESTAMP)
            "#,
        )
        .bind(&message_id)
        .bind(&local_part)
        .bind(&parsed.sender)
        .bind(recipient)
        .bind(&parsed.subject)
        .bind(&parsed.snippet)
        .bind(&parsed.body_text)
        .bind(&parsed.body_html)
        .bind(&parsed.raw_headers)
        .bind(&detected_otp)
        .bind(&detected_links)
        .execute(pool)
        .await?;

        // Handle attachments
        for att in &parsed.attachments {
            let att_id = Uuid::new_v4().to_string();
            let sanitized_filename = std::path::Path::new(&att.filename)
                .file_name()
                .and_then(|s| s.to_str())
                .filter(|s| !s.trim().is_empty())
                .unwrap_or("attachment");
            let relative_disk_path = format!("{}/{}_{}", message_id, att_id, sanitized_filename);
            let full_disk_path = attachments_dir.join(&relative_disk_path);

            if let Some(parent) = full_disk_path.parent() {
                fs::create_dir_all(parent).await?;
            }
            fs::write(&full_disk_path, &att.data).await?;

            sqlx::query(
                r#"
                INSERT INTO attachments (id, message_id, filename, content_type, size_bytes, disk_path, created_at)
                VALUES (?, ?, ?, ?, ?, ?, CURRENT_TIMESTAMP)
                "#
            )
            .bind(&att_id)
            .bind(&message_id)
            .bind(sanitized_filename)
            .bind(&att.content_type)
            .bind(att.data.len() as i64)
            .bind(&relative_disk_path)
            .execute(pool)
            .await?;
        }

        // Fetch inserted message
        let msg = sqlx::query_as::<_, Message>("SELECT * FROM messages WHERE id = ?")
            .bind(&message_id)
            .fetch_one(pool)
            .await?;

        // Broadcast realtime notification
        let _ = broadcast_tx.send(NewMailNotification {
            inbox_name: local_part.clone(),
            message_id: message_id.clone(),
            subject: parsed.subject.clone(),
            sender: parsed.sender.clone(),
        });

        saved_messages.push(msg);
    }

    Ok(saved_messages)
}

