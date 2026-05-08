use leptos::prelude::*;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <section class="todoapp">
            <header class="header">
                <h1>"todos"</h1>
                <input
                    class="new-todo"
                    placeholder="What needs to be done?"
                    autocomplete="off"
                    autofocus
                />
            </header>

            <section class="main hidden">
                <input id="toggle-all" class="toggle-all" type="checkbox" />
                <label for="toggle-all">"Mark all as complete"</label>

                <ul class="todo-list"></ul>
            </section>

            <footer class="footer hidden">
                <span class="todo-count">
                    <strong>"0"</strong>
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
