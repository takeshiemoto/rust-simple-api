use crate::repositories::RepositoryError;
use axum::async_trait;
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};

#[async_trait]
pub trait LabelRepository: Clone + Send + Sync + 'static {
    async fn create(&self, name: String) -> anyhow::Result<Label>;
    async fn all(&self) -> anyhow::Result<Vec<Label>>;
    async fn delete(&self, id: i32) -> anyhow::Result<()>;
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, FromRow)]
pub struct Label {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct UpdateLabel {
    pub id: i32,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct LabelRepositoryForDb {
    pool: PgPool,
}

impl LabelRepositoryForDb {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl LabelRepository for LabelRepositoryForDb {
    async fn create(&self, name: String) -> anyhow::Result<Label> {
        let optional_label = sqlx::query_as::<_, Label>(r#"SELECT * FROM labels WHERE name = $1"#)
            .bind(name.clone())
            .fetch_optional(&self.pool)
            .await?;
        if let Some(label) = optional_label {
            return Err(RepositoryError::Duplicate(label.id).into());
        }

        let label =
            sqlx::query_as::<_, Label>(r#"INSERT INTO labels (name) VALUES ($1) RETURNING *"#)
                .bind(name.clone())
                .fetch_one(&self.pool)
                .await?;

        Ok(label)
    }

    async fn all(&self) -> anyhow::Result<Vec<Label>> {
        todo!()
    }

    async fn delete(&self, id: i32) -> anyhow::Result<()> {
        todo!()
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

        let repository = LabelRepositoryForDb::new(pool.clone());
        let label_text = "test_label";

        // create
        let label = repository
            .create(label_text.to_string())
            .await
            .expect("[create] returned Err");

        assert_eq!(label.name, label_text);

        // all
        let labels = repository.all().await.expect("[all] returned Err");
        let label = labels.last().unwrap();

        assert_eq!(label.name, label_text);

        // delete
        repository
            .delete(label.id)
            .await
            .expect("[delete] returned Err");
    }
}
