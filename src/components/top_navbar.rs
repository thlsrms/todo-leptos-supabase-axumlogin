use leptos::*;
use leptos_router::ActionForm;

#[island]
pub fn TopNavBar(authenticated: bool) -> impl IntoView {
    let toggle_dark_mode_action = create_server_action::<ToggleDarkMode>();
    let prefers_dark = RwSignal::new(false);
    let sign_out = create_server_action::<SignOut>();
    let show_if_logged_in = RwSignal::new("none");
    let show_if_logged_out = RwSignal::new("block");

    prefers_dark.set(crate::PrefersDark::check());

    if authenticated {
        show_if_logged_in.set("block");
        show_if_logged_out.set("none");
    }

    view! {
        <div id="top-nav-bar" class="uk-position-small uk-position-top">
            <nav
                class="uk-navbar-container uk-navbar-transparent"
                uk-inverse="sel-active: .uk-navbar-transparent"
            >
                <div class=move || format!("uk-container uk-background-{0} uk-{1} bg-toggle",
            if prefers_dark() {"secondary"} else {"default"},  if prefers_dark() {"light"} else {"dark"} )
                >
                    <div uk-navbar>
                        <div class="uk-navbar-left">
                            <a
                                href="#"
                                class="uk-icon-button uk-text-warning"
                                uk-icon="bolt"
                                uk-toggle="target: .bg-toggle; cls: uk-background-secondary uk-light"
                                on:click=move |_| toggle_dark_mode_action.dispatch(ToggleDarkMode {})
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

#[server(endpoint = "toggle_dark")]
async fn toggle_dark_mode() -> Result<bool, ServerFnError> {
    use axum_extra::extract::cookie::{Cookie, SameSite};
    use axum_extra::extract::CookieJar;
    use leptos_axum::ResponseOptions;
    use time::Duration;

    let res_options = expect_context::<ResponseOptions>();
    let headers = leptos_axum::extract::<http::HeaderMap>().await?;
    let cookies = CookieJar::from_headers(&headers);
    let prefers_dark = cookies.get("dark_mode").is_some();
    let mut cookie = Cookie::build("dark_mode")
        .path("/")
        .same_site(SameSite::Lax);

    if prefers_dark {
        cookie = cookie.max_age(Duration::seconds(-1));
    }

    res_options.insert_header(
        http::header::SET_COOKIE,
        cookie.build().encoded().to_string().parse().unwrap(),
    );

    Ok(!prefers_dark)
}
