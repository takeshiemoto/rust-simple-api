use crate::repositories::{CreateTodo, TodoRepository, UpdateTodo};
use axum::extract::{Extension, FromRequest, Path, RequestParts};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{async_trait, BoxError, Json};
use rand::Rng;
use serde::de::DeserializeOwned;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;
use validator::Validate;

#[derive(Debug)]
pub struct ValidateJson<T>(T);

#[async_trait]
impl<T, B> FromRequest<B> for ValidateJson<T>
where
    T: DeserializeOwned + Validate,
    B: http_body::Body + Send,
    B::Data: Send,
    B::Error: Into<BoxError>,
{
    type Rejection = (StatusCode, String);

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req).await.map_err(|rejection| {
            let message = format!("Json parse error: [{}]", rejection);
            (StatusCode::BAD_REQUEST, message)
        })?;

        value.validate().map_err(|rejection| {
            let message = format!("Validation error: [{}]", rejection).replace('\n', ",");
            (StatusCode::BAD_REQUEST, message)
        })?;

        Ok(ValidateJson(value))
    }
}

// Extension抽出器
// アプリケーションの状態や依存関係をハンドラに注入するために使用されます。
// これにより、共有状態や他のリソースへのアクセスをハンドラ関数内で容易にできるようになります。
// create_todoでは、Extension<Arc<T>>を使用して、TodoRepositoryのインスタンスをハンドラに注入しています。
// Json(payload)では、リクエストボディをデシリアライズしてCreateTodo型に変換しています。
pub async fn create_todo<T: TodoRepository>(
    ValidateJson(payload): ValidateJson<CreateTodo>,
    Extension(repository): Extension<Arc<T>>,
) -> impl IntoResponse {
    let todo = repository.create(payload);
    (StatusCode::CREATED, Json(todo))
}

pub async fn find_todo<T: TodoRepository>(
    Path(id): Path<i32>,
    Extension(repository): Extension<Arc<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    // ok_orはOptionをErrに変換して?で即時返却している
    let todo = repository.find(id).ok_or(StatusCode::NOT_FOUND)?;
    Ok((StatusCode::OK, Json(todo)))
}

pub async fn all_todos<T: TodoRepository>(
    Extension(repository): Extension<Arc<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    let todo = repository.all();
    Ok((StatusCode::OK, Json(todo)))
}

pub async fn update_todo<T: TodoRepository>(
    Path(id): Path<i32>,
    ValidateJson(payload): ValidateJson<UpdateTodo>,
    Extension(repository): Extension<Arc<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    let todo = repository
        .update(id, payload)
        .or(Err(StatusCode::NOT_FOUND))?;
    Ok((StatusCode::OK, Json(todo)))
}

pub async fn delete_todo<T: TodoRepository>(
    Path(id): Path<i32>,
    Extension(repository): Extension<Arc<T>>,
) -> StatusCode {
    repository
        .delete(id)
        .map(|_| StatusCode::NO_CONTENT)
        .unwrap_or(StatusCode::NOT_FOUND)
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
