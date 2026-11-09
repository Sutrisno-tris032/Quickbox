use crate::connection::SmtpConnection;
use crate::ingest::NewMailNotification;
use anyhow::Result;
use sqlx::SqlitePool;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tracing::{error, info};

pub struct SmtpServer {
    listener: TcpListener,
    domain: String,
    max_bytes: usize,
    pool: Option<SqlitePool>,
    attachments_dir: Option<PathBuf>,
    broadcast_tx: Option<Arc<broadcast::Sender<NewMailNotification>>>,
}

impl SmtpServer {
    pub async fn bind(addr: &str, domain: &str, max_bytes: usize) -> Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        info!("SMTP Server bound on {}", listener.local_addr()?);
        Ok(Self {
            listener,
            domain: domain.to_string(),
            max_bytes,
            pool: None,
            attachments_dir: None,
            broadcast_tx: None,
        })
    }

    pub fn with_storage(
        mut self,
        pool: SqlitePool,
        attachments_dir: PathBuf,
        broadcast_tx: Arc<broadcast::Sender<NewMailNotification>>,
    ) -> Self {
        self.pool = Some(pool);
        self.attachments_dir = Some(attachments_dir);
        self.broadcast_tx = Some(broadcast_tx);
        self
    }

    pub fn local_addr(&self) -> Result<SocketAddr> {
        Ok(self.listener.local_addr()?)
    }

    pub async fn serve(self) -> Result<()> {
        loop {
            match self.listener.accept().await {
                Ok((stream, peer_addr)) => {
                    let domain = self.domain.clone();
                    let max_bytes = self.max_bytes;
                    let pool = self.pool.clone();
                    let attachments_dir = self.attachments_dir.clone();
                    let broadcast_tx = self.broadcast_tx.clone();

                    tokio::spawn(async move {
                        let conn = SmtpConnection::new(
                            stream,
                            peer_addr,
                            domain,
                            max_bytes,
                            pool,
                            attachments_dir,
                            broadcast_tx,
                        );
                        if let Err(e) = conn.handle().await {
                            error!("Error handling SMTP connection from {}: {}", peer_addr, e);
                        }
                    });
                }
                Err(e) => {
                    error!("Error accepting SMTP connection: {}", e);
                }
            }
        }
    }
}
