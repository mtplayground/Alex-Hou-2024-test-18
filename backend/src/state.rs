use crate::db::Db;
use crate::todo_repository::TodoRepository;

#[derive(Clone, Debug)]
pub struct AppState {
    pub db: Db,
}

impl AppState {
    pub fn new(db: Db) -> Self {
        Self { db }
    }

    pub fn todo_repository(&self) -> TodoRepository {
        TodoRepository::new(&self.db)
    }
}
