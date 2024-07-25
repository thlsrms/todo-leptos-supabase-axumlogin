use crate::error_template::{AppError, ErrorTemplate};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use super::components::TopNavBar;
use super::pages::{HomePage, SignInPage, SignUpPage};

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();

    view! {
        <Stylesheet href="https://cdn.jsdelivr.net/npm/uikit@3.21.8/dist/css/uikit.min.css"/>
        <Script src="https://cdn.jsdelivr.net/npm/uikit@3.21.8/dist/js/uikit.min.js"/>
        <Script src="https://cdn.jsdelivr.net/npm/uikit@3.21.8/dist/js/uikit-icons.min.js"/>
        <Title text="Todo - Supabase Leptos"/>
        <TopNavBar/>
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! { <ErrorTemplate outside_errors/> }.into_view()
        }>
            <main
                id="main"
                class="uk-container-expand uk-padding-large uk-text-center uk-background-default uk-dark bg-toggle"
                uk-height-viewport="expand: true"
            >
                <div uk-height-placeholder="#top-nav-bar"></div>
                <Routes>
                    <Route path="" view=HomePage/>
                    <Route path="/signin" view=SignInPage/>
                    <Route path="/signup" view=SignUpPage/>
                </Routes>
            </main>
        </Router>
    }
}
