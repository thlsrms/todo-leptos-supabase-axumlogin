#![cfg_attr(feature = "hydrate", allow(unused))]

use leptos::*;

use crate::components::TopNavBar;

#[component]
pub fn AuthProvider(#[prop(optional)] unprotected: bool, children: Children) -> impl IntoView {
    let authenticated = RwSignal::new(false);

    #[cfg(feature = "ssr")]
    {
        use crate::supabase::AuthSession;
        use axum::extract::OriginalUri;
        let user = if let Some(auth_session) = use_context::<AuthSession>() {
            authenticated.set(auth_session.user.is_some());
            auth_session.user
        } else {
            None
        };

        if let Some(OriginalUri(uri)) = use_context::<OriginalUri>() {
            if let Some(u) = user {
                if u.identity.has_mfa
                    && u.identity.aal == "aal1"
                    && uri.path() != "/user/authenticate"
                {
                    leptos_axum::redirect(&format!("/user/authenticate?cb={}", uri));
                }
            }

            if authenticated() && unprotected {
                if uri.path() == "/signin" || uri.path() == "/signup" {
                    leptos_axum::redirect("/");
                }
            } else if !authenticated() && !unprotected {
                leptos_axum::redirect("/signin");
            }
        }
    }

    view! {
      <TopNavBar authenticated=authenticated()/>
      {children()}
    }
}
