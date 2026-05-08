use leptos::prelude::*;

use crate::header::Header;
use crate::todo_item::TodoItem;
use crate::todos::TodosStore;

#[component]
pub fn App() -> impl IntoView {
    let todos = TodosStore::new();
    provide_context(todos);

    let all_todos = move || todos.todos.get();
    let has_todos = move || !all_todos().is_empty();
    let active_count = move || all_todos().iter().filter(|todo| !todo.completed).count();

    view! {
        <section class="todoapp">
            <Header />

            <section class=move || if has_todos() { "main" } else { "main hidden" }>
                <input id="toggle-all" class="toggle-all" type="checkbox" />
                <label for="toggle-all">"Mark all as complete"</label>

                <ul class="todo-list">
                    {move || {
                        all_todos()
                            .into_iter()
                            .map(|todo| view! { <TodoItem todo=todo /> })
                            .collect_view()
                    }}
                </ul>
            </section>

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
