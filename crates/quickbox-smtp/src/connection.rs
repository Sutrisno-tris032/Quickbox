use crate::ingest::{ingest_email, NewMailNotification};
use anyhow::Result;
use sqlx::SqlitePool;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::broadcast;
use tracing::{debug, error, warn};

pub struct SmtpConnection {
    stream: TcpStream,
    peer_addr: SocketAddr,
    domain: String,
    max_bytes: usize,
    pool: Option<SqlitePool>,
    attachments_dir: Option<PathBuf>,
    broadcast_tx: Option<Arc<broadcast::Sender<NewMailNotification>>>,
}

impl SmtpConnection {
    pub fn new(
        stream: TcpStream,
        peer_addr: SocketAddr,
        domain: String,
        max_bytes: usize,
        pool: Option<SqlitePool>,
        attachments_dir: Option<PathBuf>,
        broadcast_tx: Option<Arc<broadcast::Sender<NewMailNotification>>>,
    ) -> Self {
        Self {
            stream,
            peer_addr,
            domain,
            max_bytes,
            pool,
            attachments_dir,
            broadcast_tx,
        }
    }

    pub async fn handle(self) -> Result<()> {
        let (reader, mut writer) = self.stream.into_split();
        let mut reader = BufReader::new(reader);

        // Send 220 banner
        writer
            .write_all(format!("220 {} Quick-Box SMTP Service Ready\r\n", self.domain).as_bytes())
            .await?;

        let mut _mail_from: Option<String> = None;
        let mut rcpt_to: Vec<String> = Vec::new();
        let mut in_data = false;
        let mut data_buffer: Vec<u8> = Vec::new();

        let mut line = String::new();
        loop {
            line.clear();
            let bytes_read = reader.read_line(&mut line).await?;
            if bytes_read == 0 {
                break; // Connection closed by peer
            }

            if in_data {
                // Check dot termination "\r\n.\r\n" or ".\r\n" or ".\n"
                let trimmed = line.trim_end_matches(['\r', '\n']);
                if trimmed == "." {
                    in_data = false;
                    debug!("Received email data ({} bytes)", data_buffer.len());

                    // Ingest email if storage is configured
                    if let (Some(ref pool), Some(ref att_dir), Some(ref tx)) =
                        (&self.pool, &self.attachments_dir, &self.broadcast_tx)
                    {
                        if let Err(e) = ingest_email(&data_buffer, &rcpt_to, pool, att_dir, Arc::clone(tx)).await {
                            error!("Failed to ingest email: {}", e);
                        }
                    }

                    writer.write_all(b"250 2.0.0 OK: Message accepted\r\n").await?;
                    _mail_from = None;
                    rcpt_to.clear();
                    data_buffer.clear();
                } else {
                    // Check size limit
                    if data_buffer.len() + line.len() > self.max_bytes {
                        warn!("Message size limit exceeded ({} bytes)", self.max_bytes);
                        writer.write_all(b"552 5.3.4 Message size exceeds fixed limit\r\n").await?;
                        writer.flush().await?;
                        return Ok(());
                    } else {
                        // Dot un-stuffing: if line begins with "..", strip leading dot
                        let content = if line.starts_with("..") {
                            &line[1..]
                        } else {
                            &line
                        };
                        data_buffer.extend_from_slice(content.as_bytes());
                    }
                }
                continue;
            }

            let trimmed_line = line.trim();
            let upper = trimmed_line.to_uppercase();

            if upper.starts_with("EHLO") || upper.starts_with("HELO") {
                writer
                    .write_all(
                        format!(
                            "250-{} greets {}\r\n250-SIZE {}\r\n250 8BITMIME\r\n",
                            self.domain, self.peer_addr, self.max_bytes
                        )
                        .as_bytes(),
                    )
                    .await?;
            } else if upper.starts_with("MAIL FROM:") {
                let from = trimmed_line[10..].trim().trim_matches(['<', '>']).to_string();
                _mail_from = Some(from);
                writer.write_all(b"250 2.1.0 Sender OK\r\n").await?;
            } else if upper.starts_with("RCPT TO:") {
                let raw_rcpt = trimmed_line[8..].trim().trim_matches(['<', '>']).to_string();
                let clean_rcpt = raw_rcpt.to_lowercase();
                
                // Validate domain
                if let Some((_local, rcpt_domain)) = clean_rcpt.split_once('@') {
                    if crate::domain::accepts_recipient_domain(&self.domain, rcpt_domain) {
                        rcpt_to.push(clean_rcpt);
                        writer.write_all(b"250 2.1.5 Recipient OK\r\n").await?;
                    } else {
                        debug!("Rejected recipient for foreign domain: {}", clean_rcpt);
                        writer.write_all(b"550 5.1.1 Mailbox Unrecognized\r\n").await?;
                    }
                } else {
                    writer.write_all(b"501 5.1.3 Bad recipient syntax\r\n").await?;
                }
            } else if upper == "DATA" {
                if rcpt_to.is_empty() {
                    writer.write_all(b"503 5.5.1 Error: need RCPT command\r\n").await?;
                } else {
                    in_data = true;
                    writer.write_all(b"354 End data with <CR><LF>.<CR><LF>\r\n").await?;
                }
            } else if upper == "RSET" {
                _mail_from = None;
                rcpt_to.clear();
                data_buffer.clear();
                in_data = false;
                writer.write_all(b"250 2.0.0 Reset state\r\n").await?;
            } else if upper == "NOOP" {
                writer.write_all(b"250 2.0.0 OK\r\n").await?;
            } else if upper == "QUIT" {
                writer.write_all(b"221 2.0.0 Bye\r\n").await?;
                break;
            } else {
                writer.write_all(b"500 5.5.2 Command unrecognized\r\n").await?;
            }
        }

        Ok(())
    }
}
