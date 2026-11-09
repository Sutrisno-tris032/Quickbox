use crate::state::AppState;
use axum::{
    extract::{Path, State},
    response::sse::{Event, KeepAlive, Sse},
};
use futures_util::Stream;
use std::convert::Infallible;
use std::time::Duration;
use tokio_stream::wrappers::BroadcastStream;
use tokio_stream::StreamExt;

pub async fn stream_events(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let clean_name = name.trim().to_lowercase();
    let rx = state.broadcast_tx.subscribe();
    let stream = BroadcastStream::new(rx).filter_map(move |item| {
        if let Ok(notif) = item {
            if notif.inbox_name == clean_name {
                let payload = serde_json::json!({
                    "event": "new_email",
                    "id": notif.message_id,
                    "subject": notif.subject,
                    "sender": notif.sender,
                });
                return Some(Ok(Event::default().data(payload.to_string())));
            }
        }
        None
    });

    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
}
