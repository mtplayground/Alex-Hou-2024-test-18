use leptos::{ev::KeyboardEvent, prelude::*, task::spawn_local};

use crate::todos::TodosStore;

#[component]
pub fn Header() -> impl IntoView {
    let store = use_context::<TodosStore>();
    let value = RwSignal::new(String::new());
    let is_creating = RwSignal::new(false);

    let on_keydown = move |ev: KeyboardEvent| {
        if ev.key() != "Enter" || is_creating.get_untracked() {
            return;
        }

        let Some(store) = store else {
            leptos::logging::error!("todos store context is missing");
            return;
        };

        let title = value.get_untracked().trim().to_owned();

        if title.is_empty() {
            value.set(String::new());
            return;
        }

        is_creating.set(true);

        spawn_local(async move {
            match store.create_todo(title).await {
                Ok(_) => value.set(String::new()),
                Err(error) => leptos::logging::error!("failed to create todo: {error}"),
            }

            is_creating.set(false);
        });
    };

    view! {
        <header class="header">
            <h1>"todos"</h1>
            <input
                class="new-todo"
                placeholder="What needs to be done?"
                autocomplete="off"
                autofocus
                prop:value=move || value.get()
                on:input=move |ev| value.set(event_target_value(&ev))
                on:keydown=on_keydown
                disabled=move || is_creating.get()
            />
        </header>
    }
}
