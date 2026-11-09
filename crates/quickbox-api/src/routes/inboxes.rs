use crate::middleware::auth::{extract_bearer_token, generate_token, verify_token};
use crate::state::AppState;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Json},
};
use quickbox_core::models::InboxStatus;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Deserialize)]
pub struct LockRequest {
    pub password: String,
}

#[derive(Deserialize)]
pub struct UnlockRequest {
    pub password: String,
}

#[derive(Serialize)]
pub struct UnlockResponse {
    pub token: String,
}

pub async fn get_status(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<InboxStatus>, (StatusCode, Json<serde_json::Value>)> {
    let inbox = quickbox_core::db::get_or_create_inbox(&state.pool, &name)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;

    let total: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM messages WHERE inbox_name = ? AND is_deleted = 0")
        .bind(&inbox.name)
        .fetch_one(&state.pool)
        .await
        .unwrap_or((0,));

    let unread: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM messages WHERE inbox_name = ? AND is_read = 0 AND is_deleted = 0")
        .bind(&inbox.name)
        .fetch_one(&state.pool)
        .await
        .unwrap_or((0,));

    Ok(Json(InboxStatus {
        name: inbox.name,
        is_locked: inbox.is_locked,
        total_messages: total.0,
        unread_messages: unread.0,
        created_at: inbox.created_at,
        last_active_at: inbox.last_user_activity_at,
    }))
}

pub async fn lock_inbox(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(payload): Json<LockRequest>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if payload.password.trim().is_empty() {
        return Err((StatusCode::BAD_REQUEST, Json(json!({ "error": "Password cannot be empty" }))));
    }

    let hashed = quickbox_core::auth::hash_password(&payload.password)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;

    // Ensure inbox exists then lock it (lock_inbox also purges old public messages)
    quickbox_core::db::get_or_create_inbox(&state.pool, &name)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;

    quickbox_core::db::lock_inbox(&state.pool, &name, &hashed)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;

    Ok(Json(json!({ "status": "locked", "inbox": name })))
}

pub async fn unlock_inbox(
    State(state): State<AppState>,
    Path(name): Path<String>,
    Json(payload): Json<UnlockRequest>,
) -> Result<Json<UnlockResponse>, (StatusCode, Json<serde_json::Value>)> {
    let inbox = quickbox_core::db::get_or_create_inbox(&state.pool, &name)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;

    if !inbox.is_locked {
        // Not locked, return token immediately valid for 24h
        let token = generate_token(&inbox.name, &state.jwt_secret, 86400);
        return Ok(Json(UnlockResponse { token }));
    }

    let hash = inbox.password_hash.ok_or_else(|| {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": "Inbox marked locked but missing password hash" })))
    })?;

    let is_valid = quickbox_core::auth::verify_password(&hash, &payload.password)
        .unwrap_or(false);

    if !is_valid {
        return Err((StatusCode::UNAUTHORIZED, Json(json!({ "error": "Invalid password" }))));
    }

    // Touch user activity
    let _ = quickbox_core::db::touch_user_activity(&state.pool, &inbox.name).await;

    let token = generate_token(&inbox.name, &state.jwt_secret, 86400);
    Ok(Json(UnlockResponse { token }))
}

pub async fn remove_lock(
    State(state): State<AppState>,
    Path(name): Path<String>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let inbox = quickbox_core::db::get_or_create_inbox(&state.pool, &name)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;

    if inbox.is_locked {
        let auth_hdr = headers.get("Authorization").and_then(|h| h.to_str().ok());
        let token = extract_bearer_token(auth_hdr)
            .ok_or_else(|| (StatusCode::UNAUTHORIZED, Json(json!({ "error": "Missing authorization token" }))))?;

        if !verify_token(token, &inbox.name, &state.jwt_secret) {
            return Err((StatusCode::UNAUTHORIZED, Json(json!({ "error": "Invalid or expired token" }))));
        }
    }

    quickbox_core::db::remove_lock(&state.pool, &inbox.name)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;

    Ok(Json(json!({ "status": "unlocked", "inbox": name })))
}
