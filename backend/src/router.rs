use axum::{
    Json, Router,
    extract::{Path, State},
    http::{HeaderValue, Method, StatusCode},
    routing::{get, patch},
};
use serde::Serialize;
use shared::dto::{NewTodo, Todo, ToggleAll, UpdateTodo};
use tower_http::{
    cors::{AllowOrigin, Any, CorsLayer},
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
};
use tracing::Level;
use uuid::Uuid;

use crate::{config::Config, error::AppError, state::AppState};

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: &'static str,
}

pub fn build_router(state: AppState, config: &Config) -> Result<Router, AppError> {
    let cors_layer = build_cors_layer(config)?;

    Ok(Router::new()
        .route("/healthz", get(healthz))
        .route("/api/todos", get(list_todos).post(create_todo))
        .route("/api/todos/toggle-all", patch(toggle_all_todos))
        .route(
            "/api/todos/completed",
            axum::routing::delete(clear_completed_todos),
        )
        .route("/api/todos/{id}", patch(update_todo).delete(delete_todo))
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(cors_layer)
        .with_state(state))
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
    let title = validate_title(&payload.title)?;

    let todo = state.todo_repository().create(&title).await?;

    Ok((StatusCode::CREATED, Json(todo)))
}

async fn update_todo(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
    Json(payload): Json<UpdateTodo>,
) -> Result<Json<Todo>, AppError> {
    let title = match payload.title {
        Some(title) => Some(validate_title(&title)?),
        None => None,
    };

    let todo = state
        .todo_repository()
        .update_partial(id, title.as_deref(), payload.completed)
        .await?;

    todo.map(Json)
        .ok_or_else(|| AppError::NotFound(format!("todo `{id}` not found")))
}

async fn delete_todo(
    Path(id): Path<Uuid>,
    State(state): State<AppState>,
) -> Result<StatusCode, AppError> {
    let deleted = state.todo_repository().delete(id).await?;

    if !deleted {
        return Err(AppError::NotFound(format!("todo `{id}` not found")));
    }

    Ok(StatusCode::NO_CONTENT)
}

async fn toggle_all_todos(
    State(state): State<AppState>,
    Json(payload): Json<ToggleAll>,
) -> Result<Json<Vec<Todo>>, AppError> {
    state
        .todo_repository()
        .toggle_all(payload.completed)
        .await?;

    list_todos(State(state)).await
}

async fn clear_completed_todos(State(state): State<AppState>) -> Result<Json<Vec<Todo>>, AppError> {
    state.todo_repository().delete_completed().await?;

    list_todos(State(state)).await
}

fn validate_title(title: &str) -> Result<String, AppError> {
    let trimmed = title.trim();

    if trimmed.is_empty() {
        return Err(AppError::Validation("title must not be empty".to_owned()));
    }

    Ok(trimmed.to_owned())
}

fn build_cors_layer(config: &Config) -> Result<CorsLayer, AppError> {
    let allowed_origins = config
        .cors_allowed_origins
        .iter()
        .map(|origin| {
            HeaderValue::from_str(origin).map_err(|_| AppError::InvalidCorsOrigin(origin.clone()))
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(CorsLayer::new()
        .allow_methods([
            Method::GET,
            Method::POST,
            Method::PATCH,
            Method::DELETE,
            Method::OPTIONS,
        ])
        .allow_headers(Any)
        .allow_origin(AllowOrigin::list(allowed_origins)))
}
