use crate::state::AppState;
use axum::{
    body::Bytes,
    extract::State,
    http::{HeaderMap, StatusCode},
    response::Json,
};
use serde_json::json;
use tracing::{info, warn};

/// POST /api/inbound/email
///
/// Cloudflare Email Worker webhook endpoint. The Worker sends the raw RFC 822
/// email bytes as the request body with the following headers:
///   Content-Type: message/rfc822
///   X-Webhook-Secret: <WEBHOOK_SECRET env var>
///   X-Recipient: <envelope recipient, e.g. budi@quickbox.dev>
///   X-Sender: <envelope sender>
///
/// The endpoint validates the shared secret, extracts the recipient local-part
/// as the inbox name, and delegates to the same `ingest_email` function used
/// by the SMTP listener — so OTP extraction, SSE broadcast, and retention all
/// work identically regardless of ingestion path.
pub async fn inbound_email(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Json<serde_json::Value>, (StatusCode, Json<serde_json::Value>)> {
    // --- 1. Validate webhook secret ---
    if state.webhook_secret.is_empty() {
        return Err((
            StatusCode::SERVICE_UNAVAILABLE,
            Json(json!({ "error": "Webhook ingestion is not configured on this server (WEBHOOK_SECRET not set)" })),
        ));
    }

    let provided_secret = headers
        .get("X-Webhook-Secret")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    if provided_secret != state.webhook_secret {
        warn!("Webhook: received request with invalid secret");
        return Err((
            StatusCode::UNAUTHORIZED,
            Json(json!({ "error": "Invalid webhook secret" })),
        ));
    }

    // --- 2. Extract envelope recipient from header added by Cloudflare Worker ---
    let recipient_header = headers
        .get("X-Recipient")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("")
        .to_lowercase();

    // Parse local-part from "local@domain" or fall back to parsing from raw email
    let recipients: Vec<String> = if !recipient_header.is_empty() {
        vec![recipient_header.clone()]
    } else {
        // If header missing, try to parse To: from body (best-effort)
        // Note: ParsedEmail does not expose recipient — use the sender as fallback info only
        warn!("Webhook: X-Recipient header missing, cannot determine recipient, discarding email");
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "X-Recipient header is required — set it in the Cloudflare Worker" })),
        ));
    };

    if body.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(json!({ "error": "Empty email body" })),
        ));
    }

    info!(
        "Webhook: inbound email from '{}' for '{}'",
        headers.get("X-Sender").and_then(|v| v.to_str().ok()).unwrap_or("unknown"),
        recipients.join(", ")
    );

    // --- 3. Ingest using the same pipeline as SMTP ---
    let messages = quickbox_smtp::ingest::ingest_email(
        &body,
        &recipients,
        &state.pool,
        &state.attachments_dir,
        state.broadcast_tx.clone(),
    )
    .await
    .map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": format!("Ingestion failed: {}", e) })),
        )
    })?;

    info!(
        "Webhook: successfully ingested {} message(s) for {}",
        messages.len(),
        recipients.join(", ")
    );

    Ok(Json(json!({
        "status": "accepted",
        "messages_stored": messages.len(),
        "recipients": recipients,
    })))
}

