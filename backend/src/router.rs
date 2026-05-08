use axum::{
    Json, Router,
    extract::{OriginalUri, Path, State},
    http::{HeaderValue, Method, StatusCode},
    routing::{any, get, patch},
};
use serde::Serialize;
use shared::dto::{NewTodo, Todo, ToggleAll, UpdateTodo};
use tower_http::{
    cors::{AllowOrigin, Any, CorsLayer},
    services::{ServeDir, ServeFile},
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
    let api_router = Router::new()
        .route("/todos", get(list_todos).post(create_todo))
        .route("/todos/toggle-all", patch(toggle_all_todos))
        .route(
            "/todos/completed",
            axum::routing::delete(clear_completed_todos),
        )
        .route("/todos/{id}", patch(update_todo).delete(delete_todo))
        .fallback(any(api_not_found))
        .with_state(state.clone());

    let mut router = Router::new()
        .route("/healthz", get(healthz))
        .nest("/api", api_router)
        .layer(
            TraceLayer::new_for_http()
                .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                .on_request(DefaultOnRequest::new().level(Level::INFO))
                .on_response(DefaultOnResponse::new().level(Level::INFO)),
        )
        .layer(cors_layer)
        .with_state(state);

    if let Some(static_dir) = &config.static_dir {
        router = router.fallback_service(build_static_service(static_dir));
    }

    Ok(router)
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

async fn api_not_found(OriginalUri(uri): OriginalUri) -> AppError {
    AppError::NotFound(format!("route `{}` not found", uri.path()))
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

fn build_static_service(static_dir: &std::path::Path) -> ServeDir<ServeFile> {
    ServeDir::new(static_dir).fallback(ServeFile::new(static_dir.join("index.html")))
}
