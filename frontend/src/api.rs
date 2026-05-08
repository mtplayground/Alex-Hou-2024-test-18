use std::fmt;

use shared::dto::{NewTodo, Todo, ToggleAll, UpdateTodo};
use uuid::Uuid;

#[cfg(target_arch = "wasm32")]
const TODOS_ENDPOINT: &str = "/api/todos";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiError {
    pub status: Option<u16>,
    pub code: Option<String>,
    pub message: String,
}

impl ApiError {
    #[cfg(not(target_arch = "wasm32"))]
    fn unsupported_platform() -> Self {
        Self {
            status: None,
            code: Some("unsupported_platform".to_owned()),
            message: "frontend API client is only available in wasm builds".to_owned(),
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn request(message: impl Into<String>) -> Self {
        Self {
            status: None,
            code: Some("request_failed".to_owned()),
            message: message.into(),
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn decode(message: impl Into<String>) -> Self {
        Self {
            status: None,
            code: Some("decode_failed".to_owned()),
            message: message.into(),
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for ApiError {}

#[cfg(target_arch = "wasm32")]
mod imp {
    use gloo_net::http::{Request, RequestBuilder, Response};
    use serde::Deserialize;
    use serde::de::DeserializeOwned;

    use super::{ApiError, NewTodo, TODOS_ENDPOINT, Todo, ToggleAll, UpdateTodo, Uuid};

    #[derive(Debug, Deserialize)]
    struct ErrorResponse {
        error: ErrorPayload,
    }

    #[derive(Debug, Deserialize)]
    struct ErrorPayload {
        code: String,
        message: String,
    }

    pub async fn list_todos() -> Result<Vec<Todo>, ApiError> {
        send_builder_json(Request::get(TODOS_ENDPOINT)).await
    }

    pub async fn create_todo(payload: &NewTodo) -> Result<Todo, ApiError> {
        let request = Request::post(TODOS_ENDPOINT)
            .json(payload)
            .map_err(|error| {
                ApiError::request(format!("failed to serialize create_todo payload: {error}"))
            })?;

        send_request_json(request).await
    }

    pub async fn update_todo(id: Uuid, payload: &UpdateTodo) -> Result<Todo, ApiError> {
        let request = Request::patch(&todo_url(id))
            .json(payload)
            .map_err(|error| {
                ApiError::request(format!("failed to serialize update_todo payload: {error}"))
            })?;

        send_request_json(request).await
    }

    pub async fn delete_todo(id: Uuid) -> Result<(), ApiError> {
        let response = Request::delete(&todo_url(id))
            .send()
            .await
            .map_err(|error| ApiError::request(format!("delete_todo request failed: {error}")))?;

        if response.ok() {
            Ok(())
        } else {
            Err(parse_error_response(response).await)
        }
    }

    pub async fn toggle_all(payload: &ToggleAll) -> Result<Vec<Todo>, ApiError> {
        let request = Request::patch(&format!("{TODOS_ENDPOINT}/toggle-all"))
            .json(payload)
            .map_err(|error| {
                ApiError::request(format!("failed to serialize toggle_all payload: {error}"))
            })?;

        send_request_json(request).await
    }

    pub async fn clear_completed() -> Result<Vec<Todo>, ApiError> {
        send_builder_json(Request::delete(&format!("{TODOS_ENDPOINT}/completed"))).await
    }

    async fn send_builder_json<T>(request: RequestBuilder) -> Result<T, ApiError>
    where
        T: DeserializeOwned,
    {
        let response = request
            .send()
            .await
            .map_err(|error| ApiError::request(format!("request failed: {error}")))?;

        parse_json_response(response).await
    }

    async fn send_request_json<T>(request: Request) -> Result<T, ApiError>
    where
        T: DeserializeOwned,
    {
        let response = request
            .send()
            .await
            .map_err(|error| ApiError::request(format!("request failed: {error}")))?;

        parse_json_response(response).await
    }

    async fn parse_json_response<T>(response: Response) -> Result<T, ApiError>
    where
        T: DeserializeOwned,
    {
        if response.ok() {
            response.json::<T>().await.map_err(|error| {
                ApiError::decode(format!("failed to decode API response: {error}"))
            })
        } else {
            Err(parse_error_response(response).await)
        }
    }

    async fn parse_error_response(response: Response) -> ApiError {
        let status = Some(response.status());

        match response.json::<ErrorResponse>().await {
            Ok(body) => ApiError {
                status,
                code: Some(body.error.code),
                message: body.error.message,
            },
            Err(error) => ApiError {
                status,
                code: Some("unexpected_error_response".to_owned()),
                message: format!("failed to decode error response: {error}"),
            },
        }
    }

    fn todo_url(id: Uuid) -> String {
        format!("{TODOS_ENDPOINT}/{id}")
    }
}

#[cfg(not(target_arch = "wasm32"))]
mod imp {
    use super::{ApiError, NewTodo, Todo, ToggleAll, UpdateTodo, Uuid};

    pub async fn list_todos() -> Result<Vec<Todo>, ApiError> {
        Err(ApiError::unsupported_platform())
    }

    pub async fn create_todo(_payload: &NewTodo) -> Result<Todo, ApiError> {
        Err(ApiError::unsupported_platform())
    }

    pub async fn update_todo(_id: Uuid, _payload: &UpdateTodo) -> Result<Todo, ApiError> {
        Err(ApiError::unsupported_platform())
    }

    pub async fn delete_todo(_id: Uuid) -> Result<(), ApiError> {
        Err(ApiError::unsupported_platform())
    }

    pub async fn toggle_all(_payload: &ToggleAll) -> Result<Vec<Todo>, ApiError> {
        Err(ApiError::unsupported_platform())
    }

    pub async fn clear_completed() -> Result<Vec<Todo>, ApiError> {
        Err(ApiError::unsupported_platform())
    }
}

pub use imp::{clear_completed, create_todo, delete_todo, list_todos, toggle_all, update_todo};
