#[cfg(feature = "ssr")]
#[path = ""]
mod ssr {
    pub mod middlewares;

    pub mod fileserv;
    pub mod supabase;

    #[derive(axum::extract::FromRef, Clone, Debug)]
    pub struct AppState {
        pub leptos_options: leptos::LeptosOptions,
        pub supabase: std::sync::Arc<supabase::SupabaseBackend>,
    }
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

#[derive(Clone, Debug)]
pub struct PrefersDark(pub bool);

impl PrefersDark {
    fn check() -> bool {
        #[cfg(not(feature = "ssr"))]
        {
            use wasm_bindgen::{JsCast, JsValue};
            use web_sys::HtmlDocument;

            let document =
                Into::<JsValue>::into(leptos::document()).unchecked_into::<HtmlDocument>();
            let cookies = Some(document.cookie().unwrap_or_default());
            cookies.is_some_and(|c| c.contains("dark_mode"))
        }
        #[cfg(feature = "ssr")]
        {
            leptos::use_context::<Self>().is_some_and(|Self(dark_mode)| dark_mode)
        }
    }
}

#[cfg(feature = "ssr")]
use axum::extract::RawQuery;
#[cfg(feature = "ssr")]
use std::collections::HashMap;

#[cfg(feature = "ssr")]
pub fn parse_query_string(raw_query: &RawQuery) -> HashMap<String, String> {
    let mut params = HashMap::new();
    if let Some(query) = &raw_query.0 {
        for pair in query.split('&') {
            let mut split = pair.splitn(2, '=');
            let key = split.next().unwrap_or("").to_string();
            let value = split.next().unwrap_or("").to_string();
            params.insert(key, value);
        }
    }
    params
}
