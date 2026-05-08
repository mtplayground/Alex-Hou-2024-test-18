use leptos::{prelude::*, task::spawn_local};

use crate::{
    filter::{CurrentFilter, TodoFilter},
    todos::TodosStore,
};

#[component]
pub fn Footer() -> impl IntoView {
    let Some(store) = use_context::<TodosStore>() else {
        leptos::logging::error!("todos store context is missing");
        return view! { <footer class="footer hidden"></footer> }.into_any();
    };

    let Some(current_filter) = use_context::<CurrentFilter>() else {
        leptos::logging::error!("todo filter context is missing");
        return view! { <footer class="footer hidden"></footer> }.into_any();
    };

    let is_clearing = RwSignal::new(false);
    let has_todos = move || !store.todos.get().is_empty();
    let remaining_count = move || store.todos.get().iter().filter(|todo| !todo.completed).count();
    let completed_count = move || store.todos.get().iter().filter(|todo| todo.completed).count();

    view! {
        <footer class=move || if has_todos() { "footer" } else { "footer hidden" }>
            <span class="todo-count">
                <strong>{move || remaining_count().to_string()}</strong>
                {move || {
                    if remaining_count() == 1 {
                        " item left"
                    } else {
                        " items left"
                    }
                }}
            </span>

            <ul class="filters">
                <li>
                    <a
                        class=move || if current_filter.0.get() == TodoFilter::All { "selected" } else { "" }
                        href="#/"
                    >
                        "All"
                    </a>
                </li>
                <li>
                    <a
                        class=move || if current_filter.0.get() == TodoFilter::Active { "selected" } else { "" }
                        href="#/active"
                    >
                        "Active"
                    </a>
                </li>
                <li>
                    <a
                        class=move || {
                            if current_filter.0.get() == TodoFilter::Completed {
                                "selected"
                            } else {
                                ""
                            }
                        }
                        href="#/completed"
                    >
                        "Completed"
                    </a>
                </li>
            </ul>

            <button
                class=move || {
                    if completed_count() == 0 {
                        "clear-completed hidden"
                    } else {
                        "clear-completed"
                    }
                }
                disabled=move || is_clearing.get()
                on:click=move |_| {
                    if completed_count() == 0 || is_clearing.get_untracked() {
                        return;
                    }

                    is_clearing.set(true);

                    spawn_local(async move {
                        if let Err(error) = store.clear_completed().await {
                            leptos::logging::error!("failed to clear completed todos: {error}");
                        }

                        is_clearing.set(false);
                    });
                }
            >
                "Clear completed"
            </button>
        </footer>
    }
    .into_any()
}
