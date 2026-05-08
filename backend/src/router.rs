use axum::{Json, Router, extract::State, routing::get};
use serde::Serialize;

use crate::state::AppState;

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
}

pub fn build_router(state: AppState) -> Router {
    Router::new().route("/healthz", get(healthz)).with_state(state)
}

async fn healthz(State(state): State<AppState>) -> Json<HealthResponse> {
    let _ = state.db.pool();

    Json(HealthResponse { status: "ok" })
}
