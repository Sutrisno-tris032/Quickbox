use quickbox_smtp::ingest::NewMailNotification;
use sqlx::SqlitePool;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::broadcast;

#[derive(Clone)]
pub struct AppState {
    pub pool: SqlitePool,
    pub attachments_dir: PathBuf,
    pub broadcast_tx: Arc<broadcast::Sender<NewMailNotification>>,
    pub jwt_secret: String,
    /// Shared secret for Cloudflare Email Worker webhook. If empty, webhook endpoint is disabled.
    pub webhook_secret: String,
}

