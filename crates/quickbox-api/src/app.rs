use crate::routes::{events, inboxes, messages};
use crate::state::AppState;
use axum::{
    routing::{delete, get, post},
    Router,
};
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;

pub fn create_app(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    Router::new()
        .route("/api/inboxes/{name}/status", get(inboxes::get_status))
        .route("/api/inboxes/{name}/lock", post(inboxes::lock_inbox))
        .route("/api/inboxes/{name}/unlock", post(inboxes::unlock_inbox))
        .route("/api/inboxes/{name}/lock", delete(inboxes::remove_lock))
        .route("/api/inboxes/{name}/messages", get(messages::list_messages))
        .route(
            "/api/inboxes/{name}/messages/{id}",
            get(messages::get_message).delete(messages::delete_message),
        )
        .route(
            "/api/inboxes/{name}/messages/{id}/attachments",
            get(messages::list_message_attachments),
        )
        .route(
            "/api/inboxes/{name}/attachments/{id}",
            get(messages::download_attachment),
        )
        .route("/api/inboxes/{name}/events", get(events::stream_events))
        .layer(cors)
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}
