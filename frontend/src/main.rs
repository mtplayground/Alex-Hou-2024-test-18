pub mod api;
mod app;
pub mod todos;

#[cfg(target_arch = "wasm32")]
fn main() {
    leptos::mount::mount_to_body(app::App);
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    println!("frontend is a client-side wasm app; run `trunk serve`.");
}
