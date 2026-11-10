/// Menentukan apakah domain penerima diterima oleh server untuk `configured_domain` yang sedang
/// aktif. Dipakai bersama oleh listener SMTP (`RCPT TO`) dan endpoint webhook inbound HTTP agar
/// aturan penerimaan domain tidak terduplikasi dan bisa drift.
pub fn accepts_recipient_domain(configured_domain: &str, recipient_domain: &str) -> bool {
    recipient_domain == configured_domain
        || configured_domain == "localhost"
        || configured_domain == "127.0.0.1"
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accepts_matching_domain() {
        assert!(accepts_recipient_domain("quickbox.dev", "quickbox.dev"));
    }

    #[test]
    fn rejects_foreign_domain() {
        assert!(!accepts_recipient_domain("quickbox.dev", "gmail.com"));
    }

    #[test]
    fn accepts_any_domain_when_configured_as_localhost() {
        assert!(accepts_recipient_domain("localhost", "anything.example.com"));
    }

    #[test]
    fn accepts_any_domain_when_configured_as_loopback_ip() {
        assert!(accepts_recipient_domain("127.0.0.1", "anything.example.com"));
    }
}
