mod handlers;
mod repositories;

use crate::handlers::label::{all_label, create_label, delete_label};
use crate::handlers::todo::{
    all_todos, create_todo, delete_todo, find_todo, flaky, root, update_todo,
};
use crate::repositories::labels::{LabelRepository, LabelRepositoryForDb};
use crate::repositories::todo::{TodoRepository, TodoRepositoryForDb};
use axum::routing::delete;
use axum::{extract::Extension, routing::get, routing::post, Router};
use dotenv::dotenv;
use hyper::header::CONTENT_TYPE;
use sqlx::PgPool;
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer, Origin};

#[tokio::main]
async fn main() {
    let log_level = env::var("RUST_LOG").unwrap_or("info".to_string());
    env::set_var("RUST_LOG", log_level);
    tracing_subscriber::fmt::init();
    dotenv().ok();

    let database_url = &env::var("DATABASE_URL").expect("undefined [DATABASE_URL]");

    tracing::debug!("start connect database...");

    let pool = PgPool::connect(database_url)
        .await
        .unwrap_or_else(|_| panic!("fail conect database ,url is [{}]", database_url));

    let app = create_app(
        TodoRepositoryForDb::new(pool.clone()),
        LabelRepositoryForDb::new(pool.clone()),
    );
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    tracing::debug!("listening on {}", addr);

    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn create_app<Todo: TodoRepository, Label: LabelRepository>(
    todo_repository: Todo,
    label_repository: Label,
) -> Router {
    Router::new()
        .route("/", get(root))
        .route("/todos", post(create_todo::<Todo>).get(all_todos::<Todo>))
        .route(
            "/todos/:id",
            get(find_todo::<Todo>)
                .delete(delete_todo::<Todo>)
                .patch(update_todo::<Todo>),
        )
        .route(
            "/labels",
            post(create_label::<Label>).get(all_label::<Label>),
        )
        .route("labels/:id", delete(delete_label::<Label>))
        .route("/flaky", get(flaky))
        .layer(Extension(Arc::new(todo_repository)))
        .layer(Extension(Arc::new(label_repository)))
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
    use crate::repositories::labels::test_utils::LabelRepositoryForMemory;
    use crate::repositories::todo::test_utils::TodoRepositoryForMemory;
    use crate::repositories::todo::{CreateTodo, Todo};
    use axum::http::{Method, StatusCode};
    use axum::response::Response;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    #[tokio::test]
    async fn should_return_hello_world() {
        let req = Request::builder().uri("/").body(Body::empty()).unwrap();
        let res = create_app(
            TodoRepositoryForMemory::new(),
            LabelRepositoryForMemory::new(),
        )
        .oneshot(req)
        .await
        .unwrap();
        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body: String = String::from_utf8(bytes.to_vec()).unwrap();
        assert_eq!(body, "Hello, World!");
    }

    #[tokio::test]
    async fn should_created_todo() {
        let expected = Todo::new(1, "should_return_created_todo".to_string());
        let req = build_todo_req_with_json(
            "/todos",
            Method::POST,
            r#"{ "text": "should_return_created_todo" }"#.to_string(),
        );
        // oneshotは擬似リクエストを送る
        let res = create_app(
            TodoRepositoryForMemory::new(),
            LabelRepositoryForMemory::new(),
        )
        .oneshot(req)
        .await
        .unwrap();
        let todo = res_to_todo(res).await;
        assert_eq!(expected, todo);
    }

    fn build_todo_req_with_json(path: &str, method: Method, json_body: String) -> Request<Body> {
        Request::builder()
            .uri(path)
            .method(method)
            .header(CONTENT_TYPE, mime::APPLICATION_JSON.as_ref())
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

    #[tokio::test]
    async fn should_find_todo() {
        let expected = Todo::new(1, "Should_find_todo".to_string());
        let todo_repository = TodoRepositoryForMemory::new();
        let label_repository = LabelRepositoryForMemory::new();

        todo_repository
            .create(CreateTodo::new("Should_find_todo".to_string()))
            .await
            .expect("failed create todo");
        let req = build_todo_req_with_empty(Method::GET, "/todos/1");
        let res = create_app(todo_repository, label_repository)
            .oneshot(req)
            .await
            .unwrap();
        let todo = res_to_todo(res).await;
        assert_eq!(expected, todo);
    }

    #[tokio::test]
    async fn should_get_all_todos() {
        let expected = Todo::new(1, "should_get_all_todos".to_string());

        let todo_repository = TodoRepositoryForMemory::new();
        let label_repository = LabelRepositoryForMemory::new();

        todo_repository
            .create(CreateTodo::new("should_get_all_todos".to_string()))
            .await
            .expect("failed create todo");
        let req = build_todo_req_with_empty(Method::GET, "/todos");
        let res = create_app(todo_repository, label_repository)
            .oneshot(req)
            .await
            .unwrap();
        let bytes = hyper::body::to_bytes(res.into_body()).await.unwrap();
        let body: String = String::from_utf8(bytes.to_vec()).unwrap();
        let todo: Vec<Todo> = serde_json::from_str(&body)
            .unwrap_or_else(|_| panic!("cannot convert Todo instance. body: {}", body));
        assert_eq!(vec![expected], todo);
    }

    #[tokio::test]
    async fn should_update_todo() {
        let expected = Todo::new(1, "before_update_todos".to_string());
        let todo_repository = TodoRepositoryForMemory::new();
        let label_repository = LabelRepositoryForMemory::new();

        todo_repository
            .create(CreateTodo::new("before_update_todos".to_string()))
            .await
            .expect("failed create todo");
        let req = build_todo_req_with_json(
            "/todos/1",
            Method::PATCH,
            r#"{
        "id": 1,
        "text": "before_update_todos",
        "completed": false 
        }"#
            .to_string(),
        );
        let res = create_app(todo_repository, label_repository)
            .oneshot(req)
            .await
            .unwrap();
        let todo = res_to_todo(res).await;
        assert_eq!(expected, todo);
    }

    #[tokio::test]
    async fn should_delete_todo() {
        let todo_repository = TodoRepositoryForMemory::new();
        let label_repository = LabelRepositoryForMemory::new();

        todo_repository
            .create(CreateTodo::new("should_delete_todos".to_string()))
            .await
            .expect("failed create todo");
        let req = build_todo_req_with_empty(Method::DELETE, "/todos/1");
        let res = create_app(todo_repository, label_repository)
            .oneshot(req)
            .await
            .unwrap();
        assert_eq!(StatusCode::NO_CONTENT, res.status());
    }
}
