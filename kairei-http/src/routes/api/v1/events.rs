use crate::handlers::events::{emit_event, subscribe_event};
use crate::handlers::list_events;
use crate::server::AppState;
use axum::routing::get;
use axum::{Router, routing::post};

/// Create the events routes with state
pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/", get(list_events))
        .route("/{event_id}/emit", post(emit_event))
        .route("/{event_id}/subscribe", post(subscribe_event))
}
