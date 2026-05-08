use leptos::{prelude::*, task::spawn_local};

use crate::{
    filter::{CurrentFilter, TodoFilter},
    todo_item::TodoItem,
    todos::TodosStore,
};

#[component]
pub fn MainSection() -> impl IntoView {
    let Some(store) = use_context::<TodosStore>() else {
        leptos::logging::error!("todos store context is missing");
        return view! { <section class="main hidden"></section> }.into_any();
    };

    let Some(current_filter) = use_context::<CurrentFilter>() else {
        leptos::logging::error!("todo filter context is missing");
        return view! { <section class="main hidden"></section> }.into_any();
    };

    let is_toggling = RwSignal::new(false);
    let has_todos = move || !store.todos.get().is_empty();
    let all_completed = move || {
        let todos = store.todos.get();
        !todos.is_empty() && todos.iter().all(|todo| todo.completed)
    };
    let visible_todos = move || {
        store
            .todos
            .get()
            .into_iter()
            .filter(|todo| match current_filter.0.get() {
                TodoFilter::All => true,
                TodoFilter::Active => !todo.completed,
                TodoFilter::Completed => todo.completed,
            })
            .collect::<Vec<_>>()
    };

    view! {
        <section class=move || if has_todos() { "main" } else { "main hidden" }>
            <input
                id="toggle-all"
                class="toggle-all"
                type="checkbox"
                prop:checked=move || all_completed()
                disabled=move || is_toggling.get()
                on:change=move |ev| {
                    let completed = event_target_checked(&ev);

                    if is_toggling.get_untracked() {
                        return;
                    }

                    is_toggling.set(true);

                    spawn_local(async move {
                        if let Err(error) = store.toggle_all(completed).await {
                            leptos::logging::error!("failed to toggle all todos: {error}");
                        }

                        is_toggling.set(false);
                    });
                }
            />
            <label for="toggle-all">"Mark all as complete"</label>

            <ul class="todo-list">
                {move || {
                    visible_todos()
                        .into_iter()
                        .map(|todo| view! { <TodoItem todo=todo /> })
                        .collect_view()
                }}
            </ul>
        </section>
    }
    .into_any()
}
