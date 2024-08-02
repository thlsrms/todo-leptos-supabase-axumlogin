use crate::error_template::{AppError, ErrorTemplate};
use leptos::*;
use leptos_meta::*;
use leptos_router::*;

use super::pages::{
    AddNewAuthenticator, HomePage, SignInPage, SignUpPage, UserSettings, VerifyMultiFactorAuth,
};

#[component]
pub fn App() -> impl IntoView {
    // Provides context that manages stylesheets, titles, meta tags, etc.
    provide_meta_context();
    let prefers_dark = RwSignal::new(false);
    prefers_dark.set(crate::PrefersDark::check());

    view! {
      <Stylesheet href="https://cdn.jsdelivr.net/npm/uikit@3.21.8/dist/css/uikit.min.css"/>
      <Script src="https://cdn.jsdelivr.net/npm/uikit@3.21.8/dist/js/uikit.min.js"/>
      <Script src="https://cdn.jsdelivr.net/npm/uikit@3.21.8/dist/js/uikit-icons.min.js"/>
      <Title text="Todo - Supabase Leptos"/>

      <main
        id="main"
        class=&format!(
            "uk-container-expand uk-padding-large uk-text-center uk-background-{0} uk-{1} bg-toggle",
            if prefers_dark() { "secondary" } else { "default" },
            if prefers_dark() { "light" } else { "dark" },
        )

        uk-height-viewport="expand: true"
      >
        <Router fallback=|| {
            let mut outside_errors = Errors::default();
            outside_errors.insert_with_default_key(AppError::NotFound);
            view! { <ErrorTemplate outside_errors/> }.into_view()
        }>
          <div uk-height-placeholder="#top-nav-bar"></div>
          <Routes>
            <Route path="/" view=HomePage/>
            <Route path="/signin" view=SignInPage/>
            <Route path="/signup" view=SignUpPage/>
            <Route path="/user" view=UserSettings/>
            <Route path="/user/add_2fa" view=AddNewAuthenticator/>
            <Route path="/user/authenticate" view=VerifyMultiFactorAuth/>
          </Routes>
        </Router>
      </main>
    }
}
