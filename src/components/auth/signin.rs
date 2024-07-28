use super::AuthFormFields;

use leptos::*;
use leptos_router::ActionForm;

#[island]
pub fn Signin() -> impl IntoView {
    let login_action = create_server_action::<Login>();
    let oauth_connect_action = create_server_action::<ConnectOauth>();

    view! {
        <button
            class="uk-button uk-button-default uk-dark uk-box-shadow-small \
            uk-margin-top uk-inline uk-text-capitalize"
            on:click=move |_| {
                oauth_connect_action
                    .dispatch(ConnectOauth {
                        provider: "discord".into(),
                        scopes: vec!["email".into(), "identify".into()],
                    })
            }
        >
            <span uk-icon="discord" class="uk-form-icon"></span>
            <div class="uk-margin-left">"Discord Connect"</div>
        </button>

        <hr class="uk-divider-small"/>
        <ActionForm action=login_action>
            <AuthFormFields legend="" button=" Login "/>
        </ActionForm>
    }
}

#[server(prefix = "/auth", endpoint = "signin")]
async fn login(email: String, password: String) -> Result<(), ServerFnError> {
    use crate::supabase::{AuthSession, SupabaseError};
    use axum::Extension;

    let Extension(mut auth_session) = leptos_axum::extract::<Extension<AuthSession>>().await?;
    let res_options = expect_context::<leptos_axum::ResponseOptions>();

    match auth_session.authenticate((email, password)).await {
        Ok(Some(user)) => {
            if auth_session.login(&user).await.is_err() {
                res_options.set_status(http::StatusCode::INTERNAL_SERVER_ERROR);
                return Err(ServerFnError::ServerError(
                    "/signin session error - 01".to_string(),
                ));
            }
            leptos_axum::redirect("/");
            Ok(())
        }
        Ok(None) => {
            res_options.set_status(http::StatusCode::UNAUTHORIZED);
            Err(ServerFnError::new("".to_string()))
        }
        Err(e) => match e {
            axum_login::Error::Session(_) => {
                res_options.set_status(http::StatusCode::INTERNAL_SERVER_ERROR);
                Err(ServerFnError::ServerError(
                    "/signin session error - 02".to_string(),
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

#[cfg(feature = "ssr")]
fn generate_code_verifier() -> String {
    use rand::{Rng, RngCore};

    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                        abcdefghijklmnopqrstuvwxyz\
                        0123456789-._~";
    let mut buf = vec![0u8; rand::thread_rng().gen_range(43..=128)];
    rand::thread_rng().fill_bytes(&mut buf);
    let random_string: String = buf
        .iter()
        .map(|&b| CHARSET[b as usize % CHARSET.len()] as char)
        .collect();
    random_string
}

#[server(prefix = "/auth", endpoint = "connect_oauth")]
async fn connect_oauth(provider: String, scopes: Vec<String>) -> Result<(), ServerFnError> {
    use crate::supabase::Supabase;
    use axum_extra::extract::cookie::{Cookie, SameSite};
    use base64::prelude::{Engine as _, BASE64_URL_SAFE_NO_PAD};
    use leptos_axum::ResponseOptions;
    use sha2::{Digest, Sha256};
    use supabase_rust::auth::{OAuthOptions, PKCECodeChallenge};

    let code_verifier = generate_code_verifier();

    // Set the cookie to verify the `auth_code` during the callback
    let cookie = Cookie::build(("pkce_flow", code_verifier.clone()))
        .path("/")
        .http_only(true)
        .same_site(SameSite::Lax);

    expect_context::<ResponseOptions>().insert_header(
        http::header::SET_COOKIE,
        cookie.build().encoded().to_string().parse().unwrap(),
    );

    let hashed_code_verifier =
        BASE64_URL_SAFE_NO_PAD.encode(Sha256::new().chain_update(code_verifier).finalize());
    let code_challenge = PKCECodeChallenge::S256(hashed_code_verifier);

    let oauth_options = OAuthOptions {
        scopes,
        // Could be an env var : SITE_URL/auth/callback
        redirect_to: Some("http://localhost:3000/auth/callback".to_string()),
        pkce: Some(code_challenge),
        ..Default::default()
    };

    let redirect_url = expect_context::<Supabase>()
        .client
        .sign_in_oauth(&provider, oauth_options)
        .await;

    leptos_axum::redirect(&redirect_url);

    Ok(())
}

#[server(prefix = "/auth", endpoint = "callback")]
async fn callback_oauth() -> Result<(), ServerFnError> {
    use crate::supabase::{AuthSession, Supabase, SupabaseError};
    use axum::extract::Query;
    use axum::Extension;
    use axum_extra::extract::cookie::{Cookie, SameSite};
    use axum_extra::extract::CookieJar;
    use leptos_axum::ResponseOptions;
    use time::Duration;

    #[derive(serde::Deserialize, Debug)]
    struct OAuthCode {
        code: String,
    }

    let headers = leptos_axum::extract::<http::HeaderMap>().await?;
    let cookies = CookieJar::from_headers(&headers);
    let pkce_code_verifier = if let Some(cookie) = cookies.get("pkce_flow") {
        cookie.value()
    } else {
        leptos_axum::redirect("/signin");
        return Ok(());
    };

    let res_options = expect_context::<ResponseOptions>();

    let cookie = Cookie::build("pkce_flow")
        .path("/")
        .same_site(SameSite::Lax)
        .max_age(Duration::seconds(-1));

    res_options.insert_header(
        http::header::SET_COOKIE,
        cookie.build().encoded().to_string().parse().unwrap(),
    );

    let Extension(mut auth_session) = leptos_axum::extract::<Extension<AuthSession>>().await?;
    let supabase = expect_context::<Supabase>();
    let Query(OAuthCode { code }) = leptos_axum::extract::<Query<OAuthCode>>().await?;

    let user = match supabase
        .client
        .exchange_code_for_session(&code, pkce_code_verifier)
        .await
    {
        Ok(access_token) => supabase.new_session(access_token).await,
        Err(e) => Err(e),
    };

    match user {
        Ok(user) => {
            if auth_session.login(&user).await.is_err() {
                res_options.set_status(http::StatusCode::INTERNAL_SERVER_ERROR);
                return Err(ServerFnError::ServerError(
                    "/signup session error - 01".to_string(),
                ));
            }
            leptos_axum::redirect("/");
            Ok(())
        }
        Err(e) => {
            let (code, err) = SupabaseError(e).into();
            res_options.set_status(code);
            Err(err)
        }
    }
}
