use std::{env, error::Error, net::SocketAddr};

use axum::{
    Router,
    body::Body,
    http::{Request, StatusCode, header},
};
use backend::{config::Config, db::Db, router::build_router, state::AppState};
use http_body_util::BodyExt;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use shared::dto::{Todo, UpdateTodo};
use sqlx::{Executor, PgPool, postgres::PgPoolOptions};
use tower::ServiceExt;
use uuid::Uuid;

static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("./migrations");

struct TestApp {
    app: Router,
    admin_pool: PgPool,
    schema_pool: PgPool,
    schema_name: String,
}

impl TestApp {
    async fn new() -> Result<Self, Box<dyn Error>> {
        let database_url = env::var("DATABASE_URL")?;
        let admin_pool = PgPoolOptions::new()
            .max_connections(1)
            .connect(&database_url)
            .await?;
        let schema_name = format!("test_{}", Uuid::new_v4().simple());

        admin_pool
            .execute(format!(r#"CREATE SCHEMA "{schema_name}""#).as_str())
            .await?;

        let search_path = schema_name.clone();
        let schema_pool = PgPoolOptions::new()
            .max_connections(5)
            .after_connect(move |connection, _meta| {
                let search_path = search_path.clone();

                Box::pin(async move {
                    connection
                        .execute(format!(r#"SET search_path TO "{search_path}""#).as_str())
                        .await?;

                    Ok(())
                })
            })
            .connect(&database_url)
            .await?;

        MIGRATOR.run(&schema_pool).await?;

        let config = Config {
            database_url,
            bind_addr: SocketAddr::from(([127, 0, 0, 1], 0)),
            rust_log: "backend=debug".to_owned(),
            static_dir: None,
            cors_allowed_origins: vec!["http://localhost:8080".to_owned()],
        };
        let db = Db::from_pool(schema_pool.clone());
        let state = AppState::new(db);
        let app = build_router(state, &config)?;

        Ok(Self {
            app,
            admin_pool,
            schema_pool,
            schema_name,
        })
    }

    async fn cleanup(self) -> Result<(), Box<dyn Error>> {
        self.schema_pool.close().await;
        self.admin_pool
            .execute(format!(r#"DROP SCHEMA IF EXISTS "{}" CASCADE"#, self.schema_name).as_str())
            .await?;
        self.admin_pool.close().await;

        Ok(())
    }
}

fn json_request(method: &str, uri: &str, payload: Value) -> Result<Request<Body>, Box<dyn Error>> {
    Ok(Request::builder()
        .method(method)
        .uri(uri)
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(serde_json::to_vec(&payload)?))?)
}

fn empty_request(method: &str, uri: &str) -> Result<Request<Body>, Box<dyn Error>> {
    Ok(Request::builder()
        .method(method)
        .uri(uri)
        .body(Body::empty())?)
}

async fn read_json<T>(response: axum::response::Response) -> Result<T, Box<dyn Error>>
where
    T: DeserializeOwned,
{
    let bytes = response.into_body().collect().await?.to_bytes();

    Ok(serde_json::from_slice(&bytes)?)
}

async fn create_todo(app: &Router, title: &str) -> Result<Todo, Box<dyn Error>> {
    let response = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/todos",
            json!({ "title": title }),
        )?)
        .await?;

    assert_eq!(response.status(), StatusCode::CREATED);

    read_json(response).await
}

#[tokio::test]
async fn create_list_update_delete_flow() -> Result<(), Box<dyn Error>> {
    let test_app = TestApp::new().await?;
    let app = test_app.app.clone();

    let created = create_todo(&app, "  first todo  ").await?;

    assert_eq!(created.title, "first todo");
    assert!(!created.completed);
    assert_eq!(created.position, 0);

    let list_response = app
        .clone()
        .oneshot(empty_request("GET", "/api/todos")?)
        .await?;
    assert_eq!(list_response.status(), StatusCode::OK);

    let todos: Vec<Todo> = read_json(list_response).await?;
    assert_eq!(todos.len(), 1);
    assert_eq!(todos[0], created);

    let update_payload = serde_json::to_value(UpdateTodo {
        title: Some("  renamed todo  ".to_owned()),
        completed: Some(true),
    })?;
    let update_response = app
        .clone()
        .oneshot(json_request(
            "PATCH",
            &format!("/api/todos/{}", created.id),
            update_payload,
        )?)
        .await?;
    assert_eq!(update_response.status(), StatusCode::OK);

    let updated: Todo = read_json(update_response).await?;
    assert_eq!(updated.id, created.id);
    assert_eq!(updated.title, "renamed todo");
    assert!(updated.completed);
    assert_eq!(updated.position, created.position);

    let delete_response = app
        .clone()
        .oneshot(empty_request(
            "DELETE",
            &format!("/api/todos/{}", created.id),
        )?)
        .await?;
    assert_eq!(delete_response.status(), StatusCode::NO_CONTENT);

    let final_list_response = app
        .clone()
        .oneshot(empty_request("GET", "/api/todos")?)
        .await?;
    assert_eq!(final_list_response.status(), StatusCode::OK);

    let final_todos: Vec<Todo> = read_json(final_list_response).await?;
    assert!(final_todos.is_empty());

    test_app.cleanup().await?;

    Ok(())
}

#[tokio::test]
async fn toggle_all_and_clear_completed() -> Result<(), Box<dyn Error>> {
    let test_app = TestApp::new().await?;
    let app = test_app.app.clone();

    let first = create_todo(&app, "first").await?;
    let second = create_todo(&app, "second").await?;
    let third = create_todo(&app, "third").await?;

    let toggle_all_response = app
        .clone()
        .oneshot(json_request(
            "PATCH",
            "/api/todos/toggle-all",
            json!({ "completed": true }),
        )?)
        .await?;
    assert_eq!(toggle_all_response.status(), StatusCode::OK);

    let toggled: Vec<Todo> = read_json(toggle_all_response).await?;
    assert_eq!(toggled.len(), 3);
    assert_eq!(
        toggled.iter().map(|todo| todo.id).collect::<Vec<_>>(),
        vec![first.id, second.id, third.id]
    );
    assert!(toggled.iter().all(|todo| todo.completed));

    let reset_one_response = app
        .clone()
        .oneshot(json_request(
            "PATCH",
            &format!("/api/todos/{}", second.id),
            json!({ "completed": false }),
        )?)
        .await?;
    assert_eq!(reset_one_response.status(), StatusCode::OK);

    let clear_completed_response = app
        .clone()
        .oneshot(empty_request("DELETE", "/api/todos/completed")?)
        .await?;
    assert_eq!(clear_completed_response.status(), StatusCode::OK);

    let remaining: Vec<Todo> = read_json(clear_completed_response).await?;
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].id, second.id);
    assert_eq!(remaining[0].title, "second");
    assert!(!remaining[0].completed);

    test_app.cleanup().await?;

    Ok(())
}

#[tokio::test]
async fn validation_errors_return_json() -> Result<(), Box<dyn Error>> {
    let test_app = TestApp::new().await?;
    let app = test_app.app.clone();
    let created = create_todo(&app, "valid title").await?;

    let create_error_response = app
        .clone()
        .oneshot(json_request(
            "POST",
            "/api/todos",
            json!({ "title": "   " }),
        )?)
        .await?;
    assert_eq!(create_error_response.status(), StatusCode::BAD_REQUEST);

    let create_error: Value = read_json(create_error_response).await?;
    assert_eq!(create_error["error"]["code"], "validation_error");
    assert_eq!(create_error["error"]["message"], "title must not be empty");

    let update_error_response = app
        .clone()
        .oneshot(json_request(
            "PATCH",
            &format!("/api/todos/{}", created.id),
            json!({ "title": "   " }),
        )?)
        .await?;
    assert_eq!(update_error_response.status(), StatusCode::BAD_REQUEST);

    let update_error: Value = read_json(update_error_response).await?;
    assert_eq!(update_error["error"]["code"], "validation_error");
    assert_eq!(update_error["error"]["message"], "title must not be empty");

    let missing_id = Uuid::new_v4();
    let not_found_response = app
        .clone()
        .oneshot(json_request(
            "PATCH",
            &format!("/api/todos/{missing_id}"),
            json!({ "completed": true }),
        )?)
        .await?;
    assert_eq!(not_found_response.status(), StatusCode::NOT_FOUND);

    let not_found_error: Value = read_json(not_found_response).await?;
    assert_eq!(not_found_error["error"]["code"], "not_found");
    assert_eq!(
        not_found_error["error"]["message"],
        format!("todo `{missing_id}` not found")
    );

    test_app.cleanup().await?;

    Ok(())
}
