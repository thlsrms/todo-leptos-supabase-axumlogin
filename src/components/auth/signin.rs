use super::AuthFormFields;

use leptos::*;
use leptos_router::ActionForm;

#[island]
pub fn Signin() -> impl IntoView {
    let login_action = create_server_action::<Login>();

    view! {
        <ActionForm action=login_action>
            <AuthFormFields legend="Login" button=" Login "/>
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
