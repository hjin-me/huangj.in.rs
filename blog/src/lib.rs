use leptos::*;
pub mod api;
pub mod backend;
pub mod components;
pub mod error_template;
pub mod errors;
#[cfg(feature = "ssr")]
pub mod fallback;

use components::home::*;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    // _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();

    mount_to_body(|cx| {
        view! { cx,  <BlogApp/> }
    });
}
