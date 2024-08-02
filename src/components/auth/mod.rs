mod form_fields;
pub mod provider;
pub mod signin;
pub mod signup;

pub use provider::AuthProvider;

use form_fields::AuthFormFields;

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct MFAEnrollConfirm {
    pub id: String,
    pub qr_code: String, // SVG data
    pub secret: String,
    pub uri: String,
}

#[derive(serde::Serialize, serde::Deserialize, Clone, Debug, PartialEq)]
pub struct MFAFactor {
    pub id: String,
    pub name: String,
}

use leptos::*;

#[cfg(feature = "ssr")]
use crate::supabase::{AppUser, AuthSession};

#[cfg(feature = "ssr")]
pub async fn login(mut user: AppUser, mut session: AuthSession) -> Result<(), ServerFnError> {
    let supabase = expect_context::<crate::supabase::Supabase>();
    // Check if the user has verified any 2FA
    let factors = supabase
        .client
        .mfa_list_factors(user.identity.user_id.clone(), supabase.admin_token())
        .await
        .map_err(crate::supabase::map_err)?;
    let factors = filter_verified_factors(factors, &user.identity.auth_token).await;
    user.identity.has_mfa = !factors.is_empty();

    if session.login(&user).await.is_err() {
        expect_context::<leptos_axum::ResponseOptions>()
            .set_status(http::StatusCode::INTERNAL_SERVER_ERROR);
        return Err(ServerFnError::ServerError("Sign in Error".to_string()));
    }

    if factors.is_empty() {
        leptos_axum::redirect("/");
    } else {
        leptos_axum::redirect(&format!("/user/authenticate?cb={}", "/"));
    }
    Ok(())
}

#[cfg(feature = "ssr")]
pub async fn filter_verified_factors(
    factors: Vec<supabase_rust::schema::MFAFactor>,
    auth_token: &str,
) -> Vec<MFAFactor> {
    use crate::supabase::Supabase;
    use supabase_rust::schema::MFAFactorStatus;

    let supabase = expect_context::<Supabase>();

    let mut verified_factors = vec![];

    for factor in factors {
        // filter out unverified factors and delete them
        if factor.status == MFAFactorStatus::Unverified {
            let _ = supabase
                .client
                .mfa_delete_factor(factor.id, auth_token)
                .await;
        } else {
            verified_factors.push(MFAFactor {
                id: factor.id,
                // We mark the Authenticator name field as required during enrollment
                name: factor.friendly_name.unwrap(),
            });
        }
    }
    verified_factors
}
