#![cfg_attr(feature = "ssr", allow(unused))]

use leptos::*;
use serde::{Deserialize, Serialize};
use server_fn::codec::GetUrl;

#[derive(Clone, Serialize, Deserialize)]
pub struct Authentication(pub bool);

impl Authentication {
    pub fn status(&self) -> bool {
        self.0
    }
}

#[island]
pub fn AuthProvider(#[prop(optional)] unprotected: bool, children: Children) -> impl IntoView {
    let session_check = create_server_action::<CheckSession>();
    let session = create_local_resource(move || session_check.value().get(), |_| check_session());
    let (auth, set_auth) = create_signal(Authentication(false));
    provide_context(auth);

    let show_contents = move || {
        if auth().status() || unprotected {
            "block"
        } else {
            "none"
        }
    };

    view! {
        <Suspense fallback=|| ()>
            {move || {
                if let Some(s) = session() {
                    if s.is_ok() {
                        set_auth(Authentication(true));
                        if !cfg!(feature = "ssr") {
                            let pathname = window().location().pathname().unwrap();
                            if pathname == "/signin" || pathname == "/signup" {
                                window().location().set_href("/").unwrap();
                            }
                        }
                    } else {
                        set_auth(Authentication(false));
                        if !cfg!(feature = "ssr") && !unprotected {
                            window().location().set_href("/signin").unwrap();
                        }
                    }
                }
            }}

        </Suspense>
        <div style:display=show_contents>{children()}</div>
    }
}

#[cfg(feature = "ssr")]
use crate::compose_from_fn;
#[cfg(feature = "ssr")]
use crate::middlewares::require_login;

#[server(prefix = "/auth", input = GetUrl)]
#[middleware(compose_from_fn!(require_login))]
pub async fn check_session() -> Result<(), ServerFnError> {
    Ok(())
}
