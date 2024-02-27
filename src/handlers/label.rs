use crate::handlers::ValidateJson;
use crate::repositories::labels::LabelRepository;
use axum::extract::{Extension, Path};
use axum::response::IntoResponse;
use axum::Json;
use hyper::StatusCode;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use validator::Validate;

pub async fn create_label<T: LabelRepository>(
    ValidateJson(payload): ValidateJson<CreateLabel>,
    Extension(repository): Extension<Arc<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    let label = repository
        .create(payload.name)
        .await
        .or(Err(StatusCode::INTERNAL_SERVER_ERROR))?;

    Ok((StatusCode::CREATED, Json(label)))
}

pub async fn all_label<T: LabelRepository>(
    Extension(repository): Extension<Arc<T>>,
) -> Result<impl IntoResponse, StatusCode> {
    let labels = repository.all().await.unwrap();
    Ok((StatusCode::OK, Json(labels)))
}

pub async fn delete_label<T: LabelRepository>(
    Path(id): Path<i32>,
    Extension(repository): Extension<Arc<T>>,
) -> StatusCode {
    repository
        .delete(id)
        .await
        .map(|_| StatusCode::NO_CONTENT)
        .unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq, Validate)]
pub struct CreateLabel {
    #[validate(length(min = 1, message = "Can not be empty"))]
    #[validate(length(max = 100, message = "Over test length"))]
    name: String,
}
