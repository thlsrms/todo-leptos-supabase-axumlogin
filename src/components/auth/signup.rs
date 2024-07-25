use super::AuthFormFields;

use leptos::*;
use leptos_router::ActionForm;

#[island]
pub fn Signup() -> impl IntoView {
    let signup_action = create_server_action::<SignupSFn>();

    view! {
        <ActionForm action=signup_action>
            <AuthFormFields legend="New User" button="Sign up"/>
        </ActionForm>
    }
}

#[server(SignupSFn, prefix = "/auth", endpoint = "signup")]
async fn signup(email: String, password: String) -> Result<(), ServerFnError> {
    use crate::supabase::{AuthSession, Supabase, SupabaseError};
    use axum::Extension;

    let Extension(mut auth_session) = leptos_axum::extract::<Extension<AuthSession>>().await?;
    let res_options = expect_context::<leptos_axum::ResponseOptions>();
    let supabase = expect_context::<Supabase>();

    let user = match supabase
        .client
        .signup_email_password(&email, &password)
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
