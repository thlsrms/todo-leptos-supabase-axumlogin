use supabase_rust::errors::{AuthError, Error, ErrorKind, PostgrestError};

pub struct SupabaseError(pub Error);

impl From<SupabaseError> for (http::StatusCode, leptos::ServerFnError) {
    fn from(value: SupabaseError) -> (http::StatusCode, leptos::ServerFnError) {
        let value = if value.0.http_status > 499 {
            // filter_internal_error
            tracing::error!("\nInternal Error: {0:?}", value.0);
            match value.0.kind {
                ErrorKind::Auth(e) => Error {
                    http_status: value.0.http_status,
                    kind: ErrorKind::Auth(AuthError {
                        error_description: Some(
                            " Oops! Something went wrong on our end.".to_string(),
                        ),
                        msg: None,
                        ..e
                    }),
                },
                ErrorKind::Postgrest(e) => Error {
                    http_status: value.0.http_status,
                    kind: ErrorKind::Postgrest(PostgrestError {
                        code: e.code,
                        ..Default::default()
                    }),
                },
            }
        } else {
            value.0
        };

        (
            http::StatusCode::from_u16(value.http_status).unwrap(),
            match value.kind {
                ErrorKind::Auth(AuthError {
                    error_description,
                    msg,
                    ..
                }) => leptos::ServerFnError::new(
                    error_description.unwrap_or(msg.unwrap_or("".to_string())),
                ),
                ErrorKind::Postgrest(e) => leptos::ServerFnError::new(format!("{e:?}")),
            },
        )
    }
}

pub fn map_err(e: Error) -> leptos::ServerFnError {
    let (code, err) = SupabaseError(e).into();
    leptos::expect_context::<leptos_axum::ResponseOptions>().set_status(code);
    err
}
