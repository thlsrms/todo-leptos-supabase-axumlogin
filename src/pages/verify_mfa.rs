use leptos::*;
use leptos_meta::Title;
use leptos_router::ActionForm;

use crate::components::auth::AuthProvider;

#[component]
pub fn VerifyMultiFactorAuth() -> impl IntoView {
    let callback = RwSignal::new(String::default());
    #[cfg(feature = "ssr")]
    {
        use axum::extract::RawQuery;
        if let Some(rawquery) = use_context::<std::sync::Arc<RawQuery>>() {
            let queries = crate::parse_query_string(&rawquery);
            if let Some(cb) = queries.get("cb") {
                callback.set(cb.to_string());
            }
        }
    }
    view! {
      <AuthProvider>
        <Title text="Verify Authenticator - Supabase Leptos"/>
        <hr class="uk-divider-small"/>
        <div class="uk-container uk-margin-top">
          <h4 class="uk-heading-line uk-text-center">
            <span>"Two-Factor Verification"</span>
          </h4>
          <div class="uk-card uk-card-body ">
            <VerifyMFAInput callback=callback()/>
          </div>
        </div>
      </AuthProvider>
    }
}

#[island]
fn VerifyMFAInput(callback: String) -> impl IntoView {
    let code: RwSignal<String> = RwSignal::new(String::default());
    let verify_mfa_action = create_server_action::<VerifyMFA>();

    view! {
      <ActionForm action=verify_mfa_action>
        <div class="uk-flex uk-flex-column uk-flex-around uk-flex-middle">
          <input
            id="callback"
            type="text"
            class=""
            name="callback"
            value=callback
            style:display="none"
          />
          <div class="uk-form-stacked">
            <label class="uk-form-label" for="verification-code">
              "Authenticator Verification Code"
            </label>
            <input
              id="verification-code"
              type="text"
              placeholder="Verification Code"
              aria-label="Verification Code"
              class="uk-input uk-form-width-medium"
              name="code"
              on:input=move |ev| code.set(event_target_value(&ev))
              prop:value=code
              required
            />
          </div>
          <div class="uk-flex uk-flex-between uk-width-2-3">
            <button
              type="button"
              class="uk-button uk-button-default"
              on:click=move |_| {
                  window().history().unwrap().back().expect("THE TIME MACHINE BROKEN!")
              }
            >

              Cancel
            </button>
            <button type="submit" class="uk-button uk-button-primary">
              "Verify"
            </button>
          </div>
        </div>
      </ActionForm>
    }
}

#[cfg(feature = "ssr")]
#[path = ""]
mod ssr {
    pub use crate::compose_from_fn;
    pub use crate::middlewares::require_login;
}
#[cfg(feature = "ssr")]
pub use ssr::*;

#[server(name = VerifyMFA, prefix = "/auth", endpoint = "verify_mfa")]
#[middleware(compose_from_fn!(require_login))]
async fn verify_mfa(mut code: String, callback: String) -> Result<(), ServerFnError> {
    use crate::components::auth::filter_verified_factors;
    use crate::supabase::{AuthSession, Supabase};
    use axum::Extension;
    use axum_extra::extract::CookieJar;

    let Extension(auth_session) = leptos_axum::extract::<Extension<AuthSession>>().await?;
    let identity = auth_session.user.unwrap().identity;
    let supabase = expect_context::<Supabase>();

    let code: String = code.split_whitespace().collect();
    let user_factors = supabase
        .client
        .mfa_list_factors(identity.user_id.clone(), supabase.admin_token())
        .await
        .map_err(crate::supabase::map_err)?;

    let factors = filter_verified_factors(user_factors, &identity.auth_token).await;

    let access_token = {
        let mut token = None;
        for factor in &factors {
            match supabase
                .client
                .mfa_challenge_and_verify(factor.id.clone(), code.clone(), &identity.auth_token)
                .await
            {
                Ok(t) => {
                    token = Some(t);
                    break;
                }
                Err(_) => continue,
            };
        }
        token
    }
    .ok_or_else::<ServerFnError, _>(|| {
        ServerFnError::ServerError(
            "One Time Password verification failed. \
                The code didn't match any associated authenticator"
                .to_string(),
        )
    })?;

    // Update the user's session with the new access token that has an elevated AAL
    let headers = leptos_axum::extract::<http::HeaderMap>().await?;
    let cookies = CookieJar::from_headers(&headers);
    let session_id = cookies.get("auth").unwrap().value();
    supabase
        .update_assurance_level(session_id, identity.clone(), access_token)
        .await;

    leptos_axum::redirect(&callback);
    Ok(())
}
