use leptos::prelude::*;

use crate::filter::{CurrentFilter, TodoFilter};
use crate::header::Header;
use crate::main_section::MainSection;
use crate::todos::TodosStore;

#[component]
pub fn App() -> impl IntoView {
    let todos = TodosStore::new();
    let current_filter = CurrentFilter(RwSignal::new(TodoFilter::All));

    provide_context(todos);
    provide_context(current_filter);

    let all_todos = move || todos.todos.get();
    let has_todos = move || !all_todos().is_empty();
    let active_count = move || all_todos().iter().filter(|todo| !todo.completed).count();

    view! {
        <section class="todoapp">
            <Header />

            <MainSection />

            <footer class=move || if has_todos() { "footer" } else { "footer hidden" }>
                <span class="todo-count">
                    <strong>{move || active_count().to_string()}</strong>
                    " items left"
                </span>

                <ul class="filters">
                    <li>
                        <a class="selected" href="#/">
                            "All"
                        </a>
                    </li>
                    <li>
                        <a href="#/active">"Active"</a>
                    </li>
                    <li>
                        <a href="#/completed">"Completed"</a>
                    </li>
                </ul>

                <button class="clear-completed">"Clear completed"</button>
            </footer>
        </section>

        <footer class="info">
            <p>"Double-click to edit a todo"</p>
            <p>
                "Created by "
                <a href="https://todomvc.com">"TodoMVC"</a>
            </p>
            <p>"Part of " <a href="https://todomvc.com">"TodoMVC"</a></p>
        </footer>
    }
}
