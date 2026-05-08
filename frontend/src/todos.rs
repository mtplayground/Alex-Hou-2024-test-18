use leptos::prelude::*;
use shared::dto::{NewTodo, Todo, ToggleAll, UpdateTodo};
use uuid::Uuid;

use crate::api::{self, ApiError};

#[derive(Clone, Copy)]
pub struct TodosStore {
    pub todos: RwSignal<Vec<Todo>>,
    pub initial_load: LocalResource<Result<Vec<Todo>, ApiError>>,
}

impl TodosStore {
    pub fn new() -> Self {
        let todos = RwSignal::new(Vec::new());
        let initial_todos = todos;

        let initial_load = LocalResource::new(move || {
            let initial_todos = initial_todos;

            async move {
                let result = api::list_todos().await;

                if let Ok(todos) = &result {
                    initial_todos.set(todos.clone());
                }

                result
            }
        });

        Self {
            todos,
            initial_load,
        }
    }

    pub fn add(&self, todo: Todo) {
        self.todos.update(|todos| todos.push(todo));
    }

    pub fn update(&self, todo: Todo) {
        self.todos.update(|todos| {
            if let Some(existing) = todos.iter_mut().find(|existing| existing.id == todo.id) {
                *existing = todo;
            }
        });
    }

    pub fn remove(&self, id: Uuid) {
        self.todos.update(|todos| todos.retain(|todo| todo.id != id));
    }

    pub fn set_all(&self, todos: Vec<Todo>) {
        self.todos.set(todos);
    }

    pub fn refetch(&self) {
        self.initial_load.refetch();
    }

    pub async fn create_todo(&self, title: String) -> Result<Todo, ApiError> {
        let todo = api::create_todo(&NewTodo { title }).await?;
        self.add(todo.clone());
        Ok(todo)
    }

    pub async fn update_todo(
        &self,
        id: Uuid,
        title: Option<String>,
        completed: Option<bool>,
    ) -> Result<Todo, ApiError> {
        let todo = api::update_todo(id, &UpdateTodo { title, completed }).await?;
        self.update(todo.clone());
        Ok(todo)
    }

    pub async fn delete_todo(&self, id: Uuid) -> Result<(), ApiError> {
        api::delete_todo(id).await?;
        self.remove(id);
        Ok(())
    }

    pub async fn toggle_all(&self, completed: bool) -> Result<Vec<Todo>, ApiError> {
        let todos = api::toggle_all(&ToggleAll { completed }).await?;
        self.set_all(todos.clone());
        Ok(todos)
    }

    pub async fn clear_completed(&self) -> Result<Vec<Todo>, ApiError> {
        let todos = api::clear_completed().await?;
        self.set_all(todos.clone());
        Ok(todos)
    }
}
