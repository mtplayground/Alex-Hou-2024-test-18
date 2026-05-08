use leptos::prelude::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TodoFilter {
    All,
    Active,
    Completed,
}

impl TodoFilter {
    pub fn from_hash(hash: &str) -> Self {
        match hash {
            "#/active" => Self::Active,
            "#/completed" => Self::Completed,
            _ => Self::All,
        }
    }

    pub fn as_hash(self) -> &'static str {
        match self {
            Self::All => "#/",
            Self::Active => "#/active",
            Self::Completed => "#/completed",
        }
    }
}

#[derive(Clone, Copy)]
pub struct CurrentFilter(pub RwSignal<TodoFilter>);

pub fn provide_current_filter() -> CurrentFilter {
    let signal = RwSignal::new(current_hash_filter());
    let current_filter = CurrentFilter(signal);

    #[cfg(target_arch = "wasm32")]
    install_hash_change_listener(signal);

    provide_context(current_filter);

    current_filter
}

#[cfg(target_arch = "wasm32")]
fn current_hash_filter() -> TodoFilter {
    web_sys::window()
        .and_then(|window| window.location().hash().ok())
        .map(|hash| TodoFilter::from_hash(&hash))
        .unwrap_or(TodoFilter::All)
}

#[cfg(not(target_arch = "wasm32"))]
fn current_hash_filter() -> TodoFilter {
    TodoFilter::All
}

#[cfg(target_arch = "wasm32")]
fn install_hash_change_listener(signal: RwSignal<TodoFilter>) {
    use wasm_bindgen::{JsCast, closure::Closure};

    let Some(window) = web_sys::window() else {
        return;
    };

    let callback = Closure::<dyn FnMut(web_sys::Event)>::wrap(Box::new(move |_| {
        signal.set(current_hash_filter());
    }));

    if let Err(error) =
        window.add_event_listener_with_callback("hashchange", callback.as_ref().unchecked_ref())
    {
        leptos::logging::error!("failed to install hashchange listener: {error:?}");
        return;
    }

    callback.forget();
}
