#![allow(dead_code)]

use leptos::{ev::KeyboardEvent, html::Input, prelude::*, task::spawn_local};
use shared::dto::Todo;

use crate::todos::TodosStore;

#[component]
pub fn TodoItem(todo: Todo) -> impl IntoView {
    let Some(store) = use_context::<TodosStore>() else {
        leptos::logging::error!("todos store context is missing");

        return view! { <li class="todo"></li> }.into_any();
    };

    let editing = RwSignal::new(false);
    let draft = RwSignal::new(todo.title.clone());
    let edit_input = NodeRef::<Input>::new();
    let todo_id = todo.id;
    let original_title = todo.title.clone();
    let completed = todo.completed;

    Effect::new(move |_| {
        if editing.get() {
            if let Some(input) = edit_input.get() {
                let _ = input.focus();
                let cursor = input.value().len() as u32;
                let _ = input.set_selection_range(cursor, cursor);
            }
        }
    });

    view! {
        <li class=move || todo_class(completed, editing.get())>
            <div class="view">
                <input
                    class="toggle"
                    type="checkbox"
                    prop:checked=completed
                    on:change=move |ev| {
                        update_completed(todo_id, completed, event_target_checked(&ev), store);
                    }
                />

                <label
                    on:dblclick={
                        let original_title = original_title.clone();

                        move |_| {
                            draft.set(original_title.clone());
                            editing.set(true);
                        }
                    }
                >
                    {original_title.clone()}
                </label>

                <button
                    class="destroy"
                    on:click=move |_| {
                        delete_todo(todo_id, store);
                    }
                ></button>
            </div>

            <input
                node_ref=edit_input
                class="edit"
                autocomplete="off"
                spellcheck="false"
                prop:value=move || draft.get()
                on:input=move |ev| draft.set(event_target_value(&ev))
                on:blur={
                    let original_title = original_title.clone();

                    move |_| commit_edit(todo_id, original_title.clone(), draft, editing, store)
                }
                on:keydown={
                    let original_title = original_title.clone();

                    move |ev: KeyboardEvent| {
                        if ev.key() == "Enter" {
                            commit_edit(todo_id, original_title.clone(), draft, editing, store);
                        } else if ev.key() == "Escape" {
                            draft.set(original_title.clone());
                            editing.set(false);
                        }
                    }
                }
            />
        </li>
    }
    .into_any()
}

fn commit_edit(
    id: uuid::Uuid,
    original_title: String,
    draft: RwSignal<String>,
    editing: RwSignal<bool>,
    store: TodosStore,
) {
    if !editing.get_untracked() {
        return;
    }

    let trimmed = draft.get_untracked().trim().to_owned();

    if trimmed.is_empty() {
        editing.set(false);
        delete_todo(id, store);
        return;
    }

    if trimmed == original_title {
        draft.set(trimmed);
        editing.set(false);
        return;
    }

    editing.set(false);

    spawn_local(async move {
        match store.update_todo(id, Some(trimmed.clone()), None).await {
            Ok(_) => draft.set(trimmed),
            Err(error) => {
                draft.set(original_title);
                leptos::logging::error!("failed to update todo: {error}");
            }
        }
    });
}

fn delete_todo(id: uuid::Uuid, store: TodosStore) {
    spawn_local(async move {
        if let Err(error) = store.delete_todo(id).await {
            leptos::logging::error!("failed to delete todo: {error}");
        }
    });
}

fn todo_class(completed: bool, editing: bool) -> &'static str {
    match (completed, editing) {
        (true, true) => "todo completed editing",
        (true, false) => "todo completed",
        (false, true) => "todo editing",
        (false, false) => "todo",
    }
}

fn update_completed(id: uuid::Uuid, current_completed: bool, completed: bool, store: TodosStore) {
    if current_completed == completed {
        return;
    }

    spawn_local(async move {
        if let Err(error) = store.update_todo(id, None, Some(completed)).await {
            leptos::logging::error!("failed to toggle todo: {error}");
        }
    });
}
