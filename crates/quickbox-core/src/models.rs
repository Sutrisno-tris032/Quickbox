use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Inbox {
    pub name: String,
    pub is_locked: bool,
    #[serde(skip_serializing)]
    pub password_hash: Option<String>,
    pub created_at: DateTime<Utc>,
    pub last_user_activity_at: DateTime<Utc>,
    pub last_received_at: Option<DateTime<Utc>>,
    pub locked_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Message {
    pub id: String,
    pub inbox_name: String,
    pub sender: String,
    pub recipient: String,
    pub subject: String,
    pub snippet: String,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub raw_headers: String,
    pub detected_otp: Option<String>,
    pub detected_links: Option<String>,
    pub is_read: bool,
    pub is_favorite: bool,
    pub is_deleted: bool,
    pub received_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Attachment {
    pub id: String,
    pub message_id: String,
    pub filename: String,
    pub content_type: String,
    pub size_bytes: i64,
    pub disk_path: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InboxStatus {
    pub name: String,
    pub is_locked: bool,
    pub total_messages: i64,
    pub unread_messages: i64,
    pub created_at: DateTime<Utc>,
    pub last_active_at: DateTime<Utc>,
}

