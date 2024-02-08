use crate::repositories::{CreateTodo, TodoRepository};
use axum::extract::Extension;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use rand::Rng;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

pub async fn create_todo<T: TodoRepository>(
    Json(payload): Json<CreateTodo>,
    Extension(repository): Extension<Arc<T>>,
) -> impl IntoResponse {
    let todo = repository.create(payload);
    (StatusCode::CREATED, Json(todo))
}

pub async fn root() -> &'static str {
    "Hello, World!"
}

// This handler is flaky and will return a 500 Internal Server Error 50% of the time.
pub async fn flaky() -> impl IntoResponse {
    let delay = rand::thread_rng().gen_range(1..=7);
    sleep(Duration::from_secs(delay)).await;

    if rand::thread_rng().gen_bool(0.5) {
        StatusCode::INTERNAL_SERVER_ERROR
    } else {
        StatusCode::OK
    }
}
