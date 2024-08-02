use leptos::*;
use leptos_meta::Title;
use leptos_router::ActionForm;

use crate::components::auth::{AuthProvider, MFAEnrollConfirm};

#[component]
pub fn AddNewAuthenticator() -> impl IntoView {
    let mfa_enroll_action = create_server_action::<MFAEnroll>();
    let mfa_resource = create_resource(move || mfa_enroll_action.value().get(), |_| enroll_new());

    #[cfg(feature = "ssr")]
    {
        let is_error = move || mfa_resource().is_some_and(|r| r.is_err());
        if is_error() {
            leptos_axum::redirect("/user");
        }
    }

    view! {
      <AuthProvider>
        <Title text="Verify Authenticator - Supabase Leptos"/>
        <hr class="uk-divider-small"/>
        <Suspense fallback=|| {
            view! { <div uk-spinner="ratio: 4"></div> }
        }>
          {move || {
              if let Some(Ok(r)) = mfa_resource() {
                  view! {
                    <div class="uk-container uk-margin-top">
                      <SetupAuthenticator factor_id=r.id qr_code=r.qr_code secret=r.secret/>
                    </div>
                  }
              } else {
                  view! { <div uk-spinner="ratio: 4"></div> }
              }
          }}

        </Suspense>
      </AuthProvider>
    }
}

#[island]
fn SetupAuthenticator(factor_id: String, qr_code: String, secret: String) -> impl IntoView {
    let authenticator_name = RwSignal::new(String::default());
    let verification_code: RwSignal<String> = RwSignal::new(String::default());
    let mfa_verify_new_action = create_server_action::<MFAuthenticationAdd>();

    let on_submit = move |ev: ev::SubmitEvent| {
        ev.prevent_default();
        mfa_verify_new_action.dispatch(MFAuthenticationAdd {
            factor_id: factor_id.clone(),
            code: verification_code(),
            authenticator_name: authenticator_name(),
        });
        verification_code.set(String::default());
    };

    view! {
      <ActionForm action=mfa_verify_new_action on:submit=on_submit>
        <NewAuthenticatorFields
          name=authenticator_name
          code=verification_code
          qr_code=qr_code
          secret=secret
        />
      </ActionForm>
    }
}

#[component]
fn NewAuthenticatorFields(
    name: RwSignal<String>,
    code: RwSignal<String>,
    qr_code: String,
    secret: String,
) -> impl IntoView {
    view! {
      <h4 class="uk-heading-line uk-text-center">
        <span>Set up an authenticator</span>
      </h4>
      <div
        class="uk-grid uk-child-width-expand uk-grid-match uk-width-3-6 uk-margin-remove"
        uk-grid
      >
        <div class="uk-card uk-card-body">
          <p class="uk-text-default uk-text-emphasis">
            Scan the QR code with your authenticator app.
          </p>

          <div class="uk-flex uk-flex-center">
            <div class="uk-margin uk-width-auto uk-box-shadow-large">
              <div inner_html=qr_code></div>
            // <img src="data:image/png;base64,..." alt="QR Code" class="uk-border-rounded"/>
            </div>
          </div>

          <div class="uk-margin-top">
            <p>Or enter this code into your authenticator app:</p>
            <code>{secret}</code>
          </div>

        </div>

        <div class="uk-card uk-card-body ">
          <div class="uk-flex uk-flex-column uk-flex-around uk-flex-middle">
            <div class="uk-form-stacked">
              <label class="uk-form-label" for="verification-name">
                Authenticator Name
              </label>
              <input
                id="verification-code"
                name="authenticator-name"
                maxlength="30"
                type="text"
                placeholder="Authenticator Name"
                aria-label="Authenticator Name"
                class="uk-input uk-form-width-medium"
                on:input=move |ev| name.set(event_target_value(&ev))
                prop:value=name
                required
              />
            </div>
            <div class="uk-form-stacked">
              <label class="uk-form-label" for="verification-code">
                Verification Code
              </label>
              <input
                id="verification-code"
                type="text"
                placeholder="Verification Code"
                aria-label="Verification Code"
                class="uk-input uk-form-width-medium"
                on:input=move |ev| code.set(event_target_value(&ev))
                prop:value=code
                required
              />
            </div>
            <div class="uk-flex uk-flex-between uk-width-5-6">
              <button
                type="button"
                class="uk-button uk-button-default"
                on:click=move |_| {
                    window().history().unwrap().back().expect("THE TIME MACHINE BROKEN!")
                }
              >

                Cancel
              </button>
              <button class="uk-button uk-button-primary" type="submit">
                Verify
              </button>
            </div>
          </div>
        </div>
      </div>

      <hr/>
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

#[server(name = MFAEnroll, prefix = "/auth", endpoint = "enroll")]
async fn enroll_new() -> Result<MFAEnrollConfirm, ServerFnError> {
    use crate::supabase::{AuthSession, Supabase};
    use axum::Extension;
    use supabase_rust::auth::FactorType;

    let Extension(auth_session) = leptos_axum::extract::<Extension<AuthSession>>().await?;
    let supabase = expect_context::<Supabase>();

    let user_token = {
        let user = auth_session.user.unwrap();
        user.identity.auth_token
    };

    let mfa = supabase
        .client
        .mfa_enroll(None, &user_token, FactorType::TOTP)
        .await
        .map_err(|e| {
            tracing::error!("\nError during new MFA Enrollment : {e:?}");
            crate::supabase::map_err(e)
        })?;

    let decoded_svg = mfa
        .totp
        .qr_code
        .replace("\\u003c", "<")
        .replace("\\u003e", ">")
        .replace("\\\"", "\"")
        .replace("\\n", "");

    Ok(MFAEnrollConfirm {
        id: mfa.id,
        qr_code: decoded_svg,
        secret: mfa.totp.secret,
        uri: mfa.totp.uri,
    })
}

#[server(name = MFAuthenticationAdd, prefix = "/auth", endpoint = "add2fa")]
#[middleware(compose_from_fn!(require_login))]
async fn add_authenticator(
    factor_id: String,
    mut code: String,
    authenticator_name: String,
) -> Result<(), ServerFnError> {
    use crate::supabase::{AuthSession, Supabase};
    use axum::Extension;
    use axum_extra::extract::CookieJar;

    if authenticator_name.is_empty() {
        expect_context::<leptos_axum::ResponseOptions>().set_status(http::StatusCode::BAD_REQUEST);
        return Err(ServerFnError::MissingArg("authenticator_name".to_string()));
    }

    let Extension(auth_session) = leptos_axum::extract::<Extension<AuthSession>>().await?;
    let supabase = expect_context::<Supabase>();

    let mut user_identity = auth_session.user.unwrap().identity;

    let code = code.split_whitespace().collect();
    let access_token = supabase
        .client
        .mfa_challenge_and_verify(factor_id.clone(), code, &user_identity.auth_token)
        .await
        .map_err(crate::supabase::map_err)?;

    // Factor verified, update it with a friendly name
    let _ = supabase
        .client
        .mfa_update_factor(
            factor_id,
            user_identity.user_id.to_string(),
            authenticator_name,
            supabase.admin_token(),
        )
        .await
        .map_err(crate::supabase::map_err)?;

    // Update the user's session with the new access token that has an elevated AAL
    let headers = leptos_axum::extract::<http::HeaderMap>().await?;
    let cookies = CookieJar::from_headers(&headers);
    let session_id = cookies.get("auth").unwrap().value();

    user_identity.has_mfa = true;
    supabase
        .update_assurance_level(session_id, user_identity.clone(), access_token)
        .await;
    leptos_axum::redirect("/user");
    Ok(())
}
