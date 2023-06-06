use leptos::*;
use ui::home::*;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    tracing_wasm::set_as_global_default();
    // _ = console_log::init_with_level(log::Level::Debug);
    mount_to_body(|cx| {
        view! { cx,  <BlogApp/> }
    });
}
