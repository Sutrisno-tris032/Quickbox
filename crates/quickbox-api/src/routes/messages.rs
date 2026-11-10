use crate::middleware::auth::{extract_bearer_token, verify_token};
use crate::state::AppState;
use axum::{
    body::Body,
    extract::{Path, Query, State},
    http::{header, HeaderMap, StatusCode},
    response::{IntoResponse, Json, Response},
};
use quickbox_core::models::{Attachment, Message};
use serde::Deserialize;
use serde_json::json;
use tokio::fs::File;
use tokio_util::io::ReaderStream;

#[derive(Deserialize, Default)]
pub struct AttachmentQuery {
    pub token: Option<String>,
}

async fn check_inbox_access(
    state: &AppState,
    name: &str,
    headers: &HeaderMap,
    query_token: Option<&str>,
) -> Result<quickbox_core::models::Inbox, (StatusCode, Json<serde_json::Value>)> {
    let inbox = quickbox_core::db::get_or_create_inbox(&state.pool, name)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;

    if inbox.is_locked {
        let auth_hdr = headers.get("Authorization").and_then(|h| h.to_str().ok());
        let token = extract_bearer_token(auth_hdr)
            .or(query_token)
            .ok_or_else(|| (StatusCode::UNAUTHORIZED, Json(json!({ "error": "This inbox is locked. Password required." }))))?;

        if !verify_token(token, &inbox.name, &state.jwt_secret) {
            return Err((StatusCode::UNAUTHORIZED, Json(json!({ "error": "Invalid or expired session token." }))));
        }
    }

    // Touch user activity
    let _ = quickbox_core::db::touch_user_activity(&state.pool, &inbox.name).await;

    Ok(inbox)
}

pub async fn list_messages(
    State(state): State<AppState>,
    Path(name): Path<String>,
    headers: HeaderMap,
) -> Result<Json<Vec<Message>>, (StatusCode, Json<serde_json::Value>)> {
    let inbox = check_inbox_access(&state, &name, &headers, None).await?;

    let messages = sqlx::query_as::<_, Message>(
        "SELECT * FROM messages WHERE inbox_name = ? AND is_deleted = 0 ORDER BY received_at DESC LIMIT 100"
    )
    .bind(&inbox.name)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;

    Ok(Json(messages))
}

pub async fn get_message(
    State(state): State<AppState>,
    Path((name, id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<Message>, (StatusCode, Json<serde_json::Value>)> {
    let inbox = check_inbox_access(&state, &name, &headers, None).await?;

    let message = sqlx::query_as::<_, Message>(
        "SELECT * FROM messages WHERE id = ? AND inbox_name = ? AND is_deleted = 0"
    )
    .bind(&id)
    .bind(&inbox.name)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, Json(json!({ "error": "Message not found" }))))?;

    // Mark as read
    let _ = sqlx::query("UPDATE messages SET is_read = 1 WHERE id = ?")
        .bind(&id)
        .execute(&state.pool)
        .await;

    Ok(Json(message))
}

pub async fn delete_message(
    State(state): State<AppState>,
    Path((name, id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let inbox = check_inbox_access(&state, &name, &headers, None).await?;

    sqlx::query("UPDATE messages SET is_deleted = 1 WHERE id = ? AND inbox_name = ?")
        .bind(&id)
        .bind(&inbox.name)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;

    Ok(Json(json!({ "status": "deleted", "id": id })))
}

pub async fn clear_all_messages(
    State(state): State<AppState>,
    Path(name): Path<String>,
    headers: HeaderMap,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let inbox = check_inbox_access(&state, &name, &headers, None).await?;

    let result = sqlx::query("UPDATE messages SET is_deleted = 1 WHERE inbox_name = ? AND is_deleted = 0")
        .bind(&inbox.name)
        .execute(&state.pool)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;

    Ok(Json(json!({
        "status": "cleared",
        "inbox": inbox.name,
        "affected": result.rows_affected()
    })))
}

pub async fn list_message_attachments(
    State(state): State<AppState>,
    Path((name, id)): Path<(String, String)>,
    headers: HeaderMap,
) -> Result<Json<Vec<Attachment>>, (StatusCode, Json<serde_json::Value>)> {
    let _inbox = check_inbox_access(&state, &name, &headers, None).await?;

    let attachments = sqlx::query_as::<_, Attachment>(
        "SELECT a.* FROM attachments a JOIN messages m ON a.message_id = m.id WHERE a.message_id = ? AND m.inbox_name = ? ORDER BY a.created_at ASC"
    )
    .bind(&id)
    .bind(&name)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;

    Ok(Json(attachments))
}

pub async fn download_attachment(
    State(state): State<AppState>,
    Path((name, id)): Path<(String, String)>,
    Query(query): Query<AttachmentQuery>,
    headers: HeaderMap,
) -> Result<Response, (StatusCode, Json<serde_json::Value>)> {
    let _inbox = check_inbox_access(&state, &name, &headers, query.token.as_deref()).await?;

    let attachment = sqlx::query_as::<_, Attachment>(
        "SELECT a.* FROM attachments a JOIN messages m ON a.message_id = m.id WHERE a.id = ? AND m.inbox_name = ?"
    )
    .bind(&id)
    .bind(&name)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?
    .ok_or_else(|| (StatusCode::NOT_FOUND, Json(json!({ "error": "Attachment not found" }))))?;

    let full_path = state.attachments_dir.join(&attachment.disk_path);
    let file = File::open(&full_path)
        .await
        .map_err(|_| (StatusCode::NOT_FOUND, Json(json!({ "error": "Attachment file missing on disk" }))))?;

    let stream = ReaderStream::new(file);
    let body = Body::from_stream(stream);

    // Images are served inline so they can render directly inside <img> tags
    // (e.g. inline cid: images in an email body). The frontend's download links
    // already force a save-to-disk via the HTML `download` attribute, so this
    // doesn't affect the "Lampiran Berkas" download UX.
    let disposition_kind = if attachment.content_type.starts_with("image/") {
        "inline"
    } else {
        "attachment"
    };
    let content_disposition = format!("{}; filename=\"{}\"", disposition_kind, attachment.filename);

    let response = Response::builder()
        .header(header::CONTENT_TYPE, attachment.content_type)
        .header(header::CONTENT_DISPOSITION, content_disposition)
        .body(body)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(json!({ "error": e.to_string() }))))?;

    Ok(response)
}
