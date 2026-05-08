use leptos::prelude::*;

use crate::footer::Footer;
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

    view! {
        <section class="todoapp">
            <Header />

            <MainSection />

            <Footer />
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
