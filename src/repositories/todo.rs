use axum::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

use crate::repositories::labels::Label;
use crate::repositories::RepositoryError;
use validator::Validate;

// TodoRepositoryトレイトを実装する型が、Clone、Send、Syncトレイトを実装していること
// Cloneトレイとは型の値を複製する機能を提供することを示す
// Sendトレイトは、型の値がスレッド間で安全に送信できることを示す
// Syncトレイトは、型の値が複数のスレッドから参照されることが安全であることを示す
// 'staticライフタイムは、型がプログラムの実行期間中ずっと有効であることを示す
#[async_trait]
pub trait TodoRepository: Clone + Send + Sync + 'static {
    async fn create(&self, payload: CreateTodo) -> anyhow::Result<TodoEntity>;
    async fn find(&self, id: i32) -> anyhow::Result<TodoEntity>;
    async fn all(&self) -> anyhow::Result<Vec<TodoEntity>>;
    async fn update(&self, id: i32, payload: UpdateTodo) -> anyhow::Result<TodoEntity>;
    async fn delete(&self, id: i32) -> anyhow::Result<()>;
}

#[derive(Debug, Clone, PartialEq, Eq, FromRow)]
pub struct TodoWithLabelFromRow {
    id: i32,
    text: String,
    completed: bool,
    // label_id: Option<i32>,
    // label_name: Option<String>
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, FromRow)]
pub struct TodoEntity {
    id: i32,
    text: String,
    completed: bool,
    pub labels: Vec<Label>,
}

// `TodoWithLabelFromRow`型のベクターを引数として受け取り、`TodoEntity`型のベクターを返す関数
fn fold_entities(rows: Vec<TodoWithLabelFromRow>) -> Vec<TodoEntity> {
    // `rows`ベクターをイテレートし、`fold`メソッドを使って変換処理を行う
    rows.iter()
        .fold(vec![], |mut accum: Vec<TodoEntity>, current| {
            // 現在の`TodoWithLabelFromRow`オブジェクトから`TodoEntity`オブジェクトを作成し、`accum`ベクターに追加する
            accum.push(TodoEntity {
                id: current.id,
                // 現在の要素のテキストをクローン（ディープコピー）
                // cloneメソッドを使用する主な理由は、データの所有権を新しいデータ構造に移動させるか、またはデータの複製を作成する必要がある場合
                // String型のtextフィールドが所有権を持つデータ型であるためcloneが必要
                // fold_entities関数内でcurrent.textをTodoEntityのtextフィールドに直接割り当てようとすると
                // currentがrows.iter()によって借用されているため、所有権の移動が発生し、コンパイルエラーになります。
                text: current.text.clone(),
                completed: current.completed,
                labels: vec![],
            });
            accum
        })
}

fn fold_entity(row: TodoWithLabelFromRow) -> TodoEntity {
    let todo_entities = fold_entities(vec![row]);
    let todo = todo_entities.first().expect("expect 1 todo");

    todo.clone()
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Validate)]
pub struct CreateTodo {
    #[validate(length(min = 1, message = "Can not be empty"))]
    #[validate(length(max = 100, message = "Over test length"))]
    text: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Validate)]
pub struct UpdateTodo {
    #[validate(length(min = 1, message = "Can not be empty"))]
    #[validate(length(max = 100, message = "Over test length"))]
    text: Option<String>,
    completed: Option<bool>,
    labels: Option<Vec<i32>>,
}

#[derive(Debug, Clone)]
pub struct TodoRepositoryForDb {
    pool: PgPool,
}

impl TodoRepositoryForDb {
    pub fn new(pool: PgPool) -> Self {
        TodoRepositoryForDb { pool }
    }
}

#[async_trait]
impl TodoRepository for TodoRepositoryForDb {
    async fn create(&self, payload: CreateTodo) -> anyhow::Result<TodoEntity> {
        let todo = sqlx::query_as::<_, TodoWithLabelFromRow>(
            r#"INSERT INTO todos (text, completed) VALUES ($1, false) RETURNING *"#,
        )
        .bind(payload.text.clone())
        .fetch_one(&self.pool)
        .await?;

        Ok(fold_entity(todo))
    }

    async fn find(&self, id: i32) -> anyhow::Result<TodoEntity> {
        let todo = sqlx::query_as::<_, TodoWithLabelFromRow>(r#"SELECT * FROM todos WHERE id=$1"#)
            .bind(id)
            .fetch_one(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => RepositoryError::NotFound(id),
                _ => RepositoryError::Unexpected(e.to_string()),
            })?;

        Ok(fold_entity(todo))
    }

    async fn all(&self) -> anyhow::Result<Vec<TodoEntity>> {
        let todos =
            sqlx::query_as::<_, TodoWithLabelFromRow>(r#"SELECT * FROM todos ORDER BY id DESC;"#)
                .fetch_all(&self.pool)
                .await?;

        Ok(fold_entities(todos))
    }

    async fn update(&self, id: i32, payload: UpdateTodo) -> anyhow::Result<TodoEntity> {
        let old_todo = self.find(id).await?;
        let todo = sqlx::query_as::<_, TodoWithLabelFromRow>(
            r#"UPDATE TODOS SET text=$1, completed=$2 WHERE id=$3 RETURNING *"#,
        )
        .bind(payload.text.unwrap_or(old_todo.text))
        .bind(payload.completed.unwrap_or(old_todo.completed))
        .bind(id)
        .fetch_one(&self.pool)
        .await?;

        Ok(fold_entity(todo))
    }

    async fn delete(&self, id: i32) -> anyhow::Result<()> {
        sqlx::query(r#"DELETE FROM todos WHERE id=$1"#)
            .bind(id)
            .execute(&self.pool)
            .await
            .map_err(|e| match e {
                sqlx::Error::RowNotFound => RepositoryError::NotFound(id),
                _ => RepositoryError::Unexpected(e.to_string()),
            })?;

        Ok(())
    }
}
#[cfg(test)]
#[cfg(feature = "database-test")]
mod test {
    use super::*;
    use dotenv::dotenv;
    use sqlx::PgPool;
    use std::env;

    #[tokio::test]
    async fn crud_scenario() {
        dotenv().ok();

        let database_url = &env::var("DATABASE_URL").expect("undefined [DATABASE_URL]");
        let pool = PgPool::connect(database_url)
            .await
            .unwrap_or_else(|_| panic!("fail connect database, url is [{}]", database_url));

        let repository = TodoRepositoryForDb::new(pool.clone());
        let todo_text = "[crud_scenario] text";

        // create
        let created = repository
            .create(CreateTodo::new(todo_text.to_string()))
            .await
            .expect("[create] returned Err");

        assert_eq!(created.text, todo_text);
        assert!(!created.completed);

        // find
        let todo = repository
            .find(created.id)
            .await
            .expect("[find] returned Err");

        assert_eq!(created, todo);

        // all
        let todos = repository.all().await.expect("[all] returned Err");
        let todo = todos.first().unwrap();

        assert_eq!(created, *todo);

        // update
        let updated_text = "[crud_scenario] updated text";
        let todo = repository
            .update(
                todo.id,
                UpdateTodo {
                    text: Some(updated_text.to_string()),
                    completed: Some(true),
                    labels: Some(vec![]),
                },
            )
            .await
            .expect("[update] returned Err");

        assert_eq!(created.id, todo.id);
        assert_eq!(todo.text, updated_text);

        // delete
        repository
            .delete(todo.id)
            .await
            .expect("[delete] returned Err");
        let res = repository.find(created.id).await;

        assert!(res.is_err());

        let todo_rows = sqlx::query(r#"SELECT * FROM todos WHERE id=$1"#)
            .bind(todo.id)
            .fetch_all(&pool)
            .await
            .expect("[delete] todo_labels fetch error");

        assert_eq!(todo_rows.len(), 0)
    }
}

#[cfg(test)]
pub mod test_utils {
    use super::*;
    use crate::repositories::RepositoryError;
    use anyhow::Context;
    use std::collections::HashMap;
    use std::sync::{Arc, RwLock, RwLockReadGuard, RwLockWriteGuard};

    #[cfg(test)]
    impl CreateTodo {
        pub fn new(text: String) -> Self {
            Self { text }
        }
    }

    impl TodoEntity {
        pub fn new(id: i32, text: String) -> Self {
            Self {
                id,
                text,
                completed: false,
                labels: vec![],
            }
        }
    }

    type TodoDates = HashMap<i32, TodoEntity>;

    #[derive(Debug, Clone)]
    pub struct TodoRepositoryForMemory {
        store: Arc<RwLock<TodoDates>>,
        labels: Vec<Label>,
    }

    impl TodoRepositoryForMemory {
        pub fn new(labels: Vec<Label>) -> Self {
            TodoRepositoryForMemory {
                store: Arc::default(),
                labels,
            }
        }

        // HashMapに対してスレッドセーフに書き込む
        fn write_store_ref(&self) -> RwLockWriteGuard<TodoDates> {
            self.store.write().unwrap()
        }

        // HashMapからスレッドセーフに読み込む
        fn read_store_ref(&self) -> RwLockReadGuard<TodoDates> {
            self.store.read().unwrap()
        }

        fn resolve_labels(&self, labels: Vec<i32>) -> Vec<Label> {
            let mut label_list = self.labels.iter().cloned();
            let labels = labels
                .iter()
                .map(|id| label_list.find(|label| label.id == *id).unwrap())
                .collect();
            labels
        }
    }

    #[async_trait]
    impl TodoRepository for TodoRepositoryForMemory {
        async fn create(&self, payload: CreateTodo) -> anyhow::Result<TodoEntity> {
            let mut store = self.write_store_ref();
            let id = (store.len() + 1) as i32;
            let todo = TodoEntity::new(id, payload.text.clone());
            store.insert(id, todo.clone());
            Ok(todo)
        }

        async fn find(&self, id: i32) -> anyhow::Result<TodoEntity> {
            let store = self.read_store_ref();
            let todo = store
                .get(&id)
                .cloned()
                .ok_or(RepositoryError::NotFound(id))?;
            Ok(todo)
        }

        async fn all(&self) -> anyhow::Result<Vec<TodoEntity>> {
            let store = self.read_store_ref();
            Ok(Vec::from_iter(store.values().cloned()))
        }

        async fn update(&self, id: i32, payload: UpdateTodo) -> anyhow::Result<TodoEntity> {
            let mut store = self.write_store_ref();
            let todo = store.get(&id).context(RepositoryError::NotFound(id))?;
            let text = payload.text.unwrap_or(todo.text.clone());
            let completed = payload.completed.unwrap_or(todo.completed);
            let labels = match payload.labels {
                Some(label_ids) => self.resolve_labels(label_ids),
                None => todo.labels.clone(),
            };
            let todo = TodoEntity {
                id,
                text,
                completed,
                labels,
            };
            store.insert(id, todo.clone());
            Ok(todo)
        }

        async fn delete(&self, id: i32) -> anyhow::Result<()> {
            let mut store = self.write_store_ref();
            store.remove(&id).ok_or(RepositoryError::NotFound(id))?;
            Ok(())
        }
    }
}
