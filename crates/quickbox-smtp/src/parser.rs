use anyhow::Result;
use mail_parser::{MessageParser, MimeHeaders};

pub struct ParsedAttachment {
    pub filename: String,
    pub content_type: String,
    pub data: Vec<u8>,
    pub content_id: Option<String>,
}

pub struct ParsedEmail {
    pub sender: String,
    pub subject: String,
    pub snippet: String,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub raw_headers: String,
    pub attachments: Vec<ParsedAttachment>,
}

impl ParsedEmail {
    pub fn parse(raw_bytes: &[u8]) -> Result<Self> {
        let msg = MessageParser::default()
            .parse(raw_bytes)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse MIME email payload"))?;

        let sender = msg
            .from()
            .and_then(|f| f.first())
            .map(|addr| {
                if let Some(name) = addr.name() {
                    if let Some(email) = addr.address() {
                        format!("{} <{}>", name, email)
                    } else {
                        name.to_string()
                    }
                } else if let Some(email) = addr.address() {
                    email.to_string()
                } else {
                    "unknown@example.com".to_string()
                }
            })
            .unwrap_or_else(|| "unknown@example.com".to_string());

        let subject = msg.subject().unwrap_or("(No Subject)").to_string();

        let body_text = msg.body_text(0).map(|t| t.to_string());
        let body_html = msg.body_html(0).map(|h| h.to_string());

        let snippet = if let Some(ref text) = body_text {
            let clean = text.split_whitespace().collect::<Vec<_>>().join(" ");
            clean.chars().take(120).collect()
        } else if let Some(ref html) = body_html {
            let clean = html.split_whitespace().collect::<Vec<_>>().join(" ");
            clean.chars().take(120).collect()
        } else {
            String::new()
        };

        let mut attachments = Vec::new();
        for att in msg.attachments() {
            let filename = att
                .attachment_name()
                .unwrap_or("attachment")
                .to_string();
            let content_type = att
                .content_type()
                .map(|ct| match &ct.c_subtype {
                    Some(subtype) => format!("{}/{}", ct.c_type, subtype),
                    None => ct.c_type.to_string(),
                })
                .unwrap_or_else(|| "application/octet-stream".to_string());
            let data = att.contents().to_vec();
            let content_id = att
                .content_id()
                .map(|cid| cid.trim_matches(|c| c == '<' || c == '>').to_string());

            attachments.push(ParsedAttachment {
                filename,
                content_type,
                data,
                content_id,
            });
        }

        // Collect raw headers from header block
        let mut raw_headers = String::new();
        for header in msg.headers() {
            raw_headers.push_str(header.name());
            raw_headers.push_str(": ");
            if let Some(text) = header.value().as_text() {
                raw_headers.push_str(text);
            } else {
                raw_headers.push_str(&format!("{:?}", header.value()));
            }
            raw_headers.push('\n');
        }

        Ok(Self {
            sender,
            subject,
            snippet,
            body_text,
            body_html,
            raw_headers,
            attachments,
        })
    }
}
