#[cfg(feature = "ssr")]
#[path = ""]
mod ssr {
    pub mod middlewares;

    pub mod fileserv;
    pub mod supabase;
}
#[cfg(feature = "ssr")]
pub use ssr::*;

pub mod app;
mod components;
pub mod error_template;
mod pages;

#[cfg(feature = "hydrate")]
#[wasm_bindgen::prelude::wasm_bindgen]
pub fn hydrate() {
    console_error_panic_hook::set_once();
    leptos::leptos_dom::HydrationCtx::stop_hydrating();
}
