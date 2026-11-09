use regex::Regex;
use std::sync::LazyLock;

static OTP_REGEX_PROXIMITY: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"(?i)(?:code|kode|otp|verification|verifikasi|pin)[^\d\r\n]{0,25}[:\s-]?\s*(\b\d{4,8}\b)")
        .expect("Invalid OTP proximity regex")
});

static OTP_REGEX_STANDALONE: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r"\b\d{6}\b").expect("Invalid standalone 6-digit regex")
});

static URL_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(r#"https?://[^\s<>"'\)]+"#).expect("Invalid URL regex")
});

pub fn extract_otp(text: &str) -> Option<String> {
    if let Some(caps) = OTP_REGEX_PROXIMITY.captures(text) {
        if let Some(m) = caps.get(1) {
            return Some(m.as_str().to_string());
        }
    }
    // Fallback: if there's exactly one prominent 6-digit number in the text
    let mut matches = OTP_REGEX_STANDALONE.find_iter(text);
    if let Some(first) = matches.next() {
        if matches.next().is_none() {
            // Exactly one 6-digit match
            return Some(first.as_str().to_string());
        }
    }
    None
}

pub fn extract_links(text: &str) -> Vec<String> {
    URL_REGEX
        .find_iter(text)
        .map(|m| m.as_str().to_string())
        .collect()
}

