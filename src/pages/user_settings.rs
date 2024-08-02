use leptos::*;
use leptos_meta::Title;

use crate::components::auth::{AuthProvider, MFAFactor};

#[component]
pub fn UserSettings() -> impl IntoView {
    let get_user_factors = create_server_action::<ListUserMFA>();
    let user_multifactor_list = create_resource(
        move || get_user_factors.value().get(),
        |_| list_user_factors(),
    );
    let factors: RwSignal<Vec<MFAFactor>> = RwSignal::new(vec![]);

    view! {
      <Title text="User - Supabase Leptos"/>
      <AuthProvider>
        <hr class="uk-divider-small"/>
        <div class="uk-container uk-margin-top">
          <h4 class="uk-heading-line uk-text-center">
            <span>Two-Factor Authentication</span>
          </h4>

          <section class="uk-flex uk-flex-column uk-flex-middle uk-text-left">
            <div class="uk-grid-small uk-flex-middle uk-margin-bottom" uk-grid>
              <div class="uk-width-expand">
                <h4 class="uk-margin-remove-bottom">2FA Options</h4>
                <p class="uk-text-meta uk-margin-remove-top">
                  Use an app like Google Authenticator or Authy.
                </p>
              </div>
              <div class="uk-width-auto">
                <a href="/user/add_2fa" class="uk-button uk-button-small uk-button-primary">
                  "Add New"
                </a>
              </div>
            </div>

            <Transition fallback=|| {
                view! { <div uk-spinner="ratio: 2"></div> }
            }>
              {move || {
                  if let Some(Ok(f)) = user_multifactor_list() {
                      factors.set(f);
                  }
              }}
              <ul class="uk-list uk-list-divider uk-width-1-2">
                {move || {
                    if factors().is_empty() {
                        view! {
                          <>
                            <li>
                              <div class="uk-grid-small uk-flex-middle" uk-grid>
                                <div class="uk-width-expand uk-text-center">
                                  <h4 class="uk-text-default uk-margin-remove-bottom">
                                    "No Authenticator added yet"
                                  </h4>
                                </div>
                              </div>
                            </li>
                          </>
                        }
                    } else {
                        view! {
                          <>
                            <For
                              each=factors
                              key=|f| f.id.clone()
                              children=move |f: MFAFactor| {
                                  view! {
                                    <li>
                                      <div class="uk-grid-small uk-flex-middle" uk-grid>
                                        <div class="uk-width-expand">
                                          <h4 class="uk-text-default uk-margin-remove-bottom">
                                            {f.name}
                                          </h4>
                                        </div>
                                        <div class="uk-width-auto">
                                          <RemoveAuthenticatorButton factor_id=f.id.clone()/>
                                        </div>
                                      </div>
                                    </li>
                                  }
                              }
                            />
                          </>
                        }
                    }
                }}

              </ul>
            </Transition>
          </section>
        </div>
      </AuthProvider>
    }
}

#[island]
fn RemoveAuthenticatorButton(factor_id: String) -> impl IntoView {
    let prefers_dark = RwSignal::new(crate::PrefersDark::check());
    let id = factor_id.clone();
    let remove_mfa_action = create_server_action::<RemoveMFA>();

    let delete_factor = move |_| {
        remove_mfa_action.dispatch(RemoveMFA {
            factor_id: id.clone(),
        });
    };

    view! {
      {move || {
          if remove_mfa_action.version().get() > 0 {
              window().location().reload().unwrap();
          }
      }}

      <div id=&format!("confirm-delete_{}", factor_id.clone()) class="uk-flex-top" uk-modal>
        <div class=move || {
            format!(
                "uk-modal-dialog uk-modal-body uk-margin-auto-vertical uk-background-{0} uk-{1} bg-toggle",
                if prefers_dark() { "secondary" } else { "default" },
                if prefers_dark() { "light" } else { "dark" },
            )
        }>
          <h4 class="uk-modal-title uk-text-center">"Are you sure?"</h4>
          <p class="uk-text-center">
            <button class="uk-button uk-button-default uk-modal-close" type="button">
              "Cancel"
            </button>
            <button
              type="submit"
              class="uk-button uk-button-danger uk-modal-close"
              on:click=delete_factor
            >
              "Remove"
            </button>
          </p>
        </div>
      </div>

      <button
        uk-toggle=&format!("target: #confirm-delete_{factor_id}")
        class="uk-button uk-button-default"
      >
        Remove
      </button>
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

#[server(name = ListUserMFA, prefix = "/auth", endpoint = "list_factors", input = server_fn::codec::GetUrl)]
#[middleware(compose_from_fn!(require_login))]
pub async fn list_user_factors() -> Result<Vec<MFAFactor>, ServerFnError> {
    use crate::components::auth::filter_verified_factors;
    use crate::supabase::{AuthSession, Supabase};
    use axum::Extension;

    let Extension(auth_session) = leptos_axum::extract::<Extension<AuthSession>>().await?;
    let supabase = expect_context::<Supabase>();

    let (user_id, auth_token) = {
        let user = auth_session.user.unwrap();
        (user.identity.user_id, user.identity.auth_token)
    };

    let factors = supabase
        .client
        .mfa_list_factors(user_id, supabase.admin_token())
        .await
        .map_err(crate::supabase::map_err)?;

    Ok(filter_verified_factors(factors, &auth_token).await)
}

#[server(name = RemoveMFA, prefix = "/auth", endpoint = "remove_factor")]
#[middleware(compose_from_fn!(require_login))]
async fn remove_factor(factor_id: String) -> Result<(), ServerFnError> {
    use crate::components::auth::filter_verified_factors;
    use crate::supabase::{AuthSession, Supabase};
    use axum::Extension;
    use axum_extra::extract::CookieJar;

    let Extension(auth_session) = leptos_axum::extract::<Extension<AuthSession>>().await?;
    let supabase = expect_context::<Supabase>();

    let mut user_identity = auth_session.user.unwrap().identity;

    if user_identity.has_mfa && user_identity.aal == "aal1" {
        leptos_axum::redirect("/user/authenticate?cb=/user");
        return Ok(());
    }

    supabase
        .client
        .mfa_delete_factor(factor_id, &user_identity.auth_token)
        .await
        .map_err(crate::supabase::map_err)?;

    // Check if the user still has any MFA associated
    let factors = supabase
        .client
        .mfa_list_factors(user_identity.user_id.clone(), supabase.admin_token())
        .await
        .map_err(crate::supabase::map_err)?;
    let factors = filter_verified_factors(factors, &user_identity.auth_token).await;

    // Update the user's session to downgrade the session's AAL
    let headers = leptos_axum::extract::<http::HeaderMap>().await?;
    let cookies = CookieJar::from_headers(&headers);
    let session_id = cookies.get("auth").unwrap().value();

    let refreshed_token = supabase
        .client
        .refresh_token(&user_identity.refresh_token)
        .await
        .map_err(crate::supabase::map_err)?;

    user_identity.has_mfa = !factors.is_empty();
    supabase
        .update_assurance_level(session_id, user_identity.clone(), refreshed_token)
        .await;

    leptos_axum::redirect("/user");
    Ok(())
}
