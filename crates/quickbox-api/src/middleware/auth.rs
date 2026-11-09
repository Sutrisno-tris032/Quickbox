use chrono::Utc;
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

pub fn generate_token(inbox_name: &str, secret: &str, duration_secs: i64) -> String {
    let expires_at = Utc::now().timestamp() + duration_secs;
    let payload = format!("{}:{}", inbox_name.to_lowercase(), expires_at);
    let mut mac = HmacSha256::new_from_slice(secret.as_bytes()).expect("HMAC can take key of any size");
    mac.update(payload.as_bytes());
    let sig = hex::encode(mac.finalize().into_bytes());
    format!("{}.{}.{}", inbox_name.to_lowercase(), expires_at, sig)
}

pub fn verify_token(token: &str, expected_inbox: &str, secret: &str) -> bool {
    let parts: Vec<&str> = token.rsplitn(3, '.').collect();
    if parts.len() != 3 {
        return false;
    }
    // rsplitn(3, '.') yields [sig, expires_at, inbox]
    let sig = parts[0];
    let expires_at: i64 = match parts[1].parse() {
        Ok(exp) => exp,
        Err(_) => return false,
    };
    let inbox = parts[2];

    if inbox != expected_inbox.to_lowercase() {
        return false;
    }

    if Utc::now().timestamp() > expires_at {
        return false; // Expired
    }

    let payload = format!("{}:{}", inbox, expires_at);
    let mut mac = match HmacSha256::new_from_slice(secret.as_bytes()) {
        Ok(m) => m,
        Err(_) => return false,
    };
    mac.update(payload.as_bytes());
    let expected_sig = hex::encode(mac.finalize().into_bytes());

    expected_sig == sig
}

pub fn extract_bearer_token(auth_header: Option<&str>) -> Option<&str> {
    if let Some(header) = auth_header {
        if header.starts_with("Bearer ") {
            return Some(&header[7..]);
        }
    }
    None
}

