#![allow(dead_code)]

use sqlx::PgPool;
use shared::dto::Todo;
use uuid::Uuid;

use crate::{db::Db, error::AppError};

#[derive(Debug, Clone)]
pub struct TodoRepository {
    pool: PgPool,
}

impl TodoRepository {
    pub fn new(db: &Db) -> Self {
        Self {
            pool: db.pool().clone(),
        }
    }

    pub async fn list_all(&self) -> Result<Vec<Todo>, AppError> {
        let todos = sqlx::query_as!(
            Todo,
            r#"
            SELECT id, title, completed, position, created_at, updated_at
            FROM todos
            ORDER BY position ASC, created_at ASC
            "#
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(todos)
    }

    pub async fn create(&self, title: &str) -> Result<Todo, AppError> {
        let todo = sqlx::query_as!(
            Todo,
            r#"
            INSERT INTO todos (id, title, completed, position, created_at, updated_at)
            VALUES (
                $1,
                $2,
                FALSE,
                (SELECT COALESCE(MAX(position), -1) + 1 FROM todos),
                NOW(),
                NOW()
            )
            RETURNING id, title, completed, position, created_at, updated_at
            "#,
            Uuid::new_v4(),
            title
        )
        .fetch_one(&self.pool)
        .await?;

        Ok(todo)
    }

    pub async fn update_partial(
        &self,
        id: Uuid,
        title: Option<&str>,
        completed: Option<bool>,
    ) -> Result<Option<Todo>, AppError> {
        if title.is_none() && completed.is_none() {
            return self.find_by_id(id).await;
        }

        let todo = sqlx::query_as!(
            Todo,
            r#"
            UPDATE todos
            SET title = COALESCE($2, title),
                completed = COALESCE($3, completed),
                updated_at = NOW()
            WHERE id = $1
            RETURNING id, title, completed, position, created_at, updated_at
            "#,
            id,
            title,
            completed
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(todo)
    }

    pub async fn delete(&self, id: Uuid) -> Result<bool, AppError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM todos
            WHERE id = $1
            "#,
            id
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected() > 0)
    }

    pub async fn toggle_all(&self, completed: bool) -> Result<u64, AppError> {
        let result = sqlx::query!(
            r#"
            UPDATE todos
            SET completed = $1,
                updated_at = NOW()
            WHERE completed IS DISTINCT FROM $1
            "#,
            completed
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    pub async fn delete_completed(&self) -> Result<u64, AppError> {
        let result = sqlx::query!(
            r#"
            DELETE FROM todos
            WHERE completed = TRUE
            "#
        )
        .execute(&self.pool)
        .await?;

        Ok(result.rows_affected())
    }

    async fn find_by_id(&self, id: Uuid) -> Result<Option<Todo>, AppError> {
        let todo = sqlx::query_as!(
            Todo,
            r#"
            SELECT id, title, completed, position, created_at, updated_at
            FROM todos
            WHERE id = $1
            "#,
            id
        )
        .fetch_optional(&self.pool)
        .await?;

        Ok(todo)
    }
}
