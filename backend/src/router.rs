use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::get,
};
use serde::Serialize;
use shared::dto::{NewTodo, Todo};

use crate::{error::AppError, state::AppState};

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
}

pub fn build_router(state: AppState) -> Router {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/api/todos", get(list_todos).post(create_todo))
        .with_state(state)
}

async fn healthz(State(state): State<AppState>) -> Json<HealthResponse> {
    let _ = state.db.pool();

    Json(HealthResponse { status: "ok" })
}

async fn list_todos(State(state): State<AppState>) -> Result<Json<Vec<Todo>>, AppError> {
    let todos = state.todo_repository().list_all().await?;

    Ok(Json(todos))
}

async fn create_todo(
    State(state): State<AppState>,
    Json(payload): Json<NewTodo>,
) -> Result<(StatusCode, Json<Todo>), AppError> {
    let title = payload.title.trim();

    if title.is_empty() {
        return Err(AppError::Validation(
            "title must not be empty".to_owned(),
        ));
    }

    let todo = state.todo_repository().create(title).await?;

    Ok((StatusCode::CREATED, Json(todo)))
}
