use crate::repositories::{CreateTodo, TodoRepository};
use axum::extract::{Extension, Path};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::Json;
use rand::Rng;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

// Extension抽出器
// アプリケーションの状態や依存関係をハンドラに注入するために使用されます。
// これにより、共有状態や他のリソースへのアクセスをハンドラ関数内で容易にできるようになります。
// create_todoでは、Extension<Arc<T>>を使用して、TodoRepositoryのインスタンスをハンドラに注入しています。
// Json(payload)では、リクエストボディをデシリアライズしてCreateTodo型に変換しています。
pub async fn create_todo<T: TodoRepository>(
    Json(payload): Json<CreateTodo>,
    Extension(repository): Extension<Arc<T>>,
) -> impl IntoResponse {
    let todo = repository.create(payload);
    (StatusCode::CREATED, Json(todo))
}

pub async fn find_todo<T: TodoRepository>(
    Path(id): Path<i32>,
    Extension(repository): Extension<Arc<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    Ok(StatusCode::OK)
}

pub async fn all_todos<T: TodoRepository>(
    Extension(repository): Extension<Arc<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    todo!();
    Ok(StatusCode::OK)
}

pub async fn update_todo<T: TodoRepository>(
    Path(id): Path<i32>,
    Json(payload): Json<CreateTodo>,
    Extension(repository): Extension<Arc<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    todo!();
    Ok(StatusCode::OK)
}

pub async fn delete_todo<T: TodoRepository>(
    Path(id): Path<i32>,
    Extension(repository): Extension<Arc<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    todo!();
    Ok(StatusCode::OK)
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
