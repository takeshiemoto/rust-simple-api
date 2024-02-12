mod handlers;
mod repositories;

use crate::handlers::{all_todos, create_todo, delete_todo, find_todo, flaky, root, update_todo};
use crate::repositories::{TodoRepository, TodoRepositoryForMemory};
use axum::{extract::Extension, routing::get, routing::post, Router};
use hyper::header::CONTENT_TYPE;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer, Origin};

#[tokio::main]
async fn main() {
    let log_level = env::var("RUST_LOG").unwrap_or("info".to_string());
    env::set_var("RUST_LOG", log_level);
    tracing_subscriber::fmt::init();

    let repository = TodoRepositoryForMemory::new();
    let app = create_app(repository);
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn create_app<T: TodoRepository>(repository: T) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/todos", post(create_todo::<T>).get(all_todos::<T>))
        .route(
            "/todos/:id",
            get(find_todo::<T>)
                .delete(delete_todo::<T>)
                .patch(update_todo::<T>),
        )
        .route("/flaky", get(flaky))
        .layer(Extension(Arc::new(repository)))
        .layer(
            CorsLayer::new()
                .allow_origin(Origin::exact("http://localhost:5173".parse().unwrap()))
                .allow_methods(Any)
                .allow_headers(vec![CONTENT_TYPE]),
        )
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::repositories::{Todo, TodoRepositoryForMemory};
    use axum::http::{header, Method};
    use axum::response::Response;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    #[tokio::test]
    async fn should_return_hello_world() {
        let repository = TodoRepositoryForMemory::new();
        let req = Request::builder().uri("/").body(Body::empty()).unwrap();
        let res = create_app(repository).oneshot(req).await.unwrap();
        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body: String = String::from_utf8(bytes.to_vec()).unwrap();
        assert_eq!(body, "Hello, World!");
    }

    #[tokio::test]
    async fn should_created_todo() {
        let expected = Todo::new(1, "should_return_created_todo".to_string());
        let repository = TodoRepositoryForMemory::new();
        let req = build_todo_req_with_json(
            "/todos",
            Method::POST,
            r#"{ "text": "should_return_created_todo" }"#.to_string(),
        );
        // oneshotは擬似リクエストを送る
        let res = create_app(repository).oneshot(req).await.unwrap();
        let todo = res_to_todo(res).await;
        assert_eq!(expected, todo);
    }

    fn build_todo_req_with_json(path: &str, method: Method, json_body: String) -> Request<Body> {
        Request::builder()
            .uri(path)
            .method(method)
            .header(header::CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
            .body(Body::from(json_body))
            .unwrap()
    }
    fn build_todo_req_with_empty(method: Method, path: &str) -> Request<Body> {
        Request::builder()
            .uri(path)
            .method(method)
            .body(Body::empty())
            .unwrap()
    }

    async fn res_to_todo(res: Response) -> Todo {
        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body: String = String::from_utf8(bytes.to_vec()).unwrap();
        let todo: Todo = serde_json::from_str(&body)
            .expect(&format!("cannot convert Todo instance. body: {}", body));
        todo
    }
}
