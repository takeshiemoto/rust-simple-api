use axum::extract::{FromRequest, RequestParts};
use axum::http::StatusCode;
use axum::{async_trait, BoxError, Json};
use serde::de::DeserializeOwned;
use validator::Validate;

pub mod label;
pub mod todo;

// ジェネリック型 `T` をラップするタプル構造体。
#[derive(Debug)]
pub struct ValidateJson<T>(T);

// `FromRequest` トレイトを `ValidateJson<T>` 構造体のために非同期で実装。
// この実装は、HTTP リクエストから `ValidateJson<T>` インスタンスを生成する方法を提供。
#[async_trait]
impl<T, B> FromRequest<B> for ValidateJson<T>
where
    // `T` は `DeserializeOwned` と `Validate` トレイトを実装する必要がある。
    // これは、`T` がデシリアライズ可能で、かつバリデーションができることを意味する。
    T: DeserializeOwned + Validate,
    // `B` は HTTP ボディを表す型で、`Send` トレイトを実装している必要がある。
    // これにより、`B` 型の値がスレッド間で安全に送信できることが保証される。
    B: http_body::Body + Send,
    // `B::Data` と `B::Error` も `Send` トレイトを実装する必要がある。
    // これにより、ボディのデータやエラーもスレッド間で安全に送信できる。
    B::Data: Send,
    B::Error: Into<BoxError>,
{
    // リクエストからの変換が失敗した場合に返されるエラーの型を定義。
    type Rejection = (StatusCode, String);

    // `from_request` は、HTTP リクエストから `ValidateJson<T>` インスタンスを生成。
    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        // `Json::<T>` の `from_request` 関数を呼び出してリクエストから値をデシリアライズし、
        // 失敗した場合はエラーメッセージを設定して `BAD_REQUEST` ステータスを返す。
        let Json(value) = Json::<T>::from_request(req).await.map_err(|rejection| {
            let message = format!("Json parse error: [{}]", rejection);
            (StatusCode::BAD_REQUEST, message)
        })?;

        // デシリアライズされた値に対してバリデーションを実行し、
        // 失敗した場合はエラーメッセージを設定して `BAD_REQUEST` ステータスを返す。
        value.validate().map_err(|rejection| {
            let message = format!("Validation error: [{}]", rejection).replace('\n', ",");
            (StatusCode::BAD_REQUEST, message)
        })?;

        // バリデーションに成功した場合、`ValidateJson(value)` を `Ok` でラップして返す。
        Ok(ValidateJson(value))
    }
}
