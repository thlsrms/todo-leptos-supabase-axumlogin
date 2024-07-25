use leptos::*;
use leptos_router::ActionForm;

use crate::components::auth::provider::{check_session, CheckSession};

#[island]
pub fn TopNavBar() -> impl IntoView {
    let sign_out = create_server_action::<SignOut>();
    let session_check = create_server_action::<CheckSession>();
    let session = create_local_resource(move || session_check.value().get(), |_| check_session());
    let (show_if_logged_in, set_logged_in) = create_signal("none");
    let (show_if_logged_out, set_logged_out) = create_signal("block");

    view! {
        <Suspense fallback=|| ()>
            {move || {
                if let Some(s) = session() {
                    if s.is_ok() {
                        set_logged_in("block");
                        set_logged_out("none");
                    } else {
                        set_logged_in("none");
                        set_logged_out("block");
                    }
                }
            }}

        </Suspense>
        <div id="top-nav-bar" class="uk-position-small uk-position-top bg-toggle">
            <nav
                class="uk-navbar-container uk-navbar-transparent"
                uk-inverse="sel-active: .uk-navbar-transparent"
            >
                <div class="uk-container">
                    <div uk-navbar>
                        <div class="uk-navbar-left">
                            <a
                                href="#"
                                class="uk-icon-button uk-text-warning"
                                uk-icon="bolt"
                                uk-toggle="target: .bg-toggle; cls: uk-background-secondary uk-light"
                            ></a>
                        </div>
                        <div class="uk-navbar-center">
                            <a href="/" class="uk-navbar-item uk-logo">
                                "Tasks"
                            </a>
                        </div>
                        <div class="uk-navbar-right">
                            <ul class="uk-navbar-nav">
                                <div style:display=show_if_logged_out>
                                    <li>
                                        <a
                                            href="/signin"
                                            class="uk-navbar-item uk-text-primary uk-button uk-button-link"
                                        >
                                            <span
                                                class="uk-icon uk-margin-small-left"
                                                uk-icon="sign-in"
                                            ></span>
                                            "Sign in"
                                        </a>
                                    </li>
                                </div>
                                <div style:display=show_if_logged_in>
                                    <li>
                                        <ActionForm action=sign_out>
                                            <button
                                                type="submit"
                                                class="uk-navbar-item uk-text-primary uk-button uk-button-link"
                                            >
                                                "Sign out"
                                                <span
                                                    class="uk-icon uk-margin-small-right"
                                                    uk-icon="sign-out"
                                                ></span>
                                            </button>
                                        </ActionForm>
                                    </li>
                                </div>
                            </ul>
                        </div>
                    </div>
                </div>
            </nav>
        </div>
    }
}

#[cfg(feature = "ssr")]
use crate::middlewares::require_login;

#[cfg(feature = "ssr")]
use crate::compose_from_fn;

#[server(prefix = "/auth", endpoint = "signout")]
#[middleware(compose_from_fn!(require_login))]
async fn sign_out() -> Result<(), ServerFnError> {
    use crate::supabase::{AuthSession, Supabase, SupabaseError};
    use axum::Extension;

    let Extension(mut auth_session) = leptos_axum::extract::<Extension<AuthSession>>().await?;
    let res_options = expect_context::<leptos_axum::ResponseOptions>();
    let supabase = expect_context::<Supabase>();

    leptos_axum::redirect("/signin");
    match auth_session.logout().await {
        Ok(u) => {
            let Some(user) = u else {
                return Err(ServerFnError::ServerError(
                    "/signout session error - 01".to_string(),
                ));
            };
            match supabase.client.logout(user.identity.auth_token).await {
                Ok(_) => Ok(()),
                Err(e) => {
                    let (code, err) = SupabaseError(e).into();
                    res_options.set_status(code);
                    Err(err)
                }
            }
        }

        Err(e) => match e {
            axum_login::Error::Session(e) => {
                tracing::error!("Error signing out : {e:?}");
                res_options.set_status(http::StatusCode::INTERNAL_SERVER_ERROR);
                Err(ServerFnError::ServerError(
                    "/signout session error - 02".to_string(),
                ))
            }
            axum_login::Error::Backend(e) => {
                let (code, err) = SupabaseError(e).into();
                res_options.set_status(code);
                Err(err)
            }
        },
    }
}
