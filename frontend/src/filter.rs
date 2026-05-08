use leptos::prelude::RwSignal;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TodoFilter {
    All,
    Active,
    Completed,
}

#[derive(Clone, Copy)]
pub struct CurrentFilter(pub RwSignal<TodoFilter>);
