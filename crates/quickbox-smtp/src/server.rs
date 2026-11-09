use crate::connection::SmtpConnection;
use anyhow::Result;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tracing::{error, info};

pub struct SmtpServer {
    listener: TcpListener,
    domain: String,
    max_bytes: usize,
}

impl SmtpServer {
    pub async fn bind(addr: &str, domain: &str, max_bytes: usize) -> Result<Self> {
        let listener = TcpListener::bind(addr).await?;
        info!("SMTP Server bound on {}", listener.local_addr()?);
        Ok(Self {
            listener,
            domain: domain.to_string(),
            max_bytes,
        })
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
                    tokio::spawn(async move {
                        let conn = SmtpConnection::new(stream, peer_addr, domain, max_bytes);
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
