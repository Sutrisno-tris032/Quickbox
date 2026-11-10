use crate::state::AppState;
use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
};
use subtle::ConstantTimeEq;
use tracing::error;

/// Perbandingan string yang aman terhadap timing attack (lihat CLAUDE.md: "perbandingan sandi
/// memakai fungsi yang aman terhadap timing attack"). Panjang berbeda langsung dianggap tidak
/// cocok tanpa membandingkan isi byte demi byte.
fn constant_time_eq(a: &str, b: &str) -> bool {
    let (a, b) = (a.as_bytes(), b.as_bytes());
    a.len() == b.len() && bool::from(a.ct_eq(b))
}

/// Endpoint internal yang dipanggil oleh Cloudflare Email Worker untuk meneruskan email mentah
/// (`message/rfc822`) yang diterima lewat Cloudflare Email Routing. Dinonaktifkan (404) kecuali
/// `INBOUND_SECRET` diset.
pub async fn inbound(State(state): State<AppState>, headers: HeaderMap, body: Bytes) -> StatusCode {
    let Some(configured_secret) = state.inbound_secret.as_deref() else {
        return StatusCode::NOT_FOUND;
    };

    let provided_secret = headers
        .get("x-quickbox-secret")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");
    if !constant_time_eq(provided_secret, configured_secret) {
        return StatusCode::UNAUTHORIZED;
    }

    let recipient = headers
        .get("x-quickbox-rcpt")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.trim().to_lowercase())
        .filter(|v| !v.is_empty());

    let Some(recipient) = recipient else {
        return StatusCode::UNPROCESSABLE_ENTITY;
    };

    let valid_recipient = recipient
        .split_once('@')
        .map(|(_local, rcpt_domain)| {
            quickbox_smtp::domain::accepts_recipient_domain(&state.domain, rcpt_domain)
        })
        .unwrap_or(false);

    if !valid_recipient {
        return StatusCode::UNPROCESSABLE_ENTITY;
    }

    match quickbox_smtp::ingest::ingest_email(
        &body,
        &[recipient],
        &state.pool,
        &state.attachments_dir,
        state.broadcast_tx.clone(),
    )
    .await
    {
        Ok(_) => StatusCode::ACCEPTED,
        Err(e) => {
            error!("Gagal mengolah email dari webhook inbound: {e}");
            StatusCode::INTERNAL_SERVER_ERROR
        }
    }
}
