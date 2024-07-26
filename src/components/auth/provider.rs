use leptos::*;

use crate::components::TopNavBar;

#[component]
pub fn AuthProvider(#[prop(optional)] unprotected: bool, children: Children) -> impl IntoView {
    let authenticated = RwSignal::new(false);

    #[cfg(feature = "ssr")]
    {
        use crate::supabase::AuthSession;
        use axum::extract::OriginalUri;
        if let Some(auth_session) = use_context::<AuthSession>() {
            authenticated.set(auth_session.user.is_some());
        } else {
            authenticated.set(false);
        }

        if let Some(OriginalUri(uri)) = use_context::<OriginalUri>() {
            if authenticated() && unprotected {
                if uri == "/signin" || uri == "/signup" {
                    leptos_axum::redirect("/");
                }
            } else if !authenticated() && !unprotected {
                leptos_axum::redirect("/signin");
            }
        }
    }

    let show_contents = move || {
        if authenticated() || unprotected {
            "block"
        } else {
            "none"
        }
    };

    view! {
        <TopNavBar authenticated=authenticated() />
        <div style:display=show_contents>{children()}</div>
    }
}
