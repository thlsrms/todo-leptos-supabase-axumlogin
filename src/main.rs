#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use std::sync::Arc;

    use axum::body::Body;
    use axum::extract::{Request, State};
    use axum::response::{IntoResponse, Response};
    use axum::routing::get;
    use axum::{Extension, Router};
    use axum_extra::extract::CookieJar;
    use axum_login::tower_sessions::SessionManagerLayer;
    use axum_login::AuthManagerLayerBuilder;
    use leptos::*;
    use leptos_axum::{generate_route_list, handle_server_fns_with_context, LeptosRoutes};
    use todo_leptos_supabase::app::*;
    use todo_leptos_supabase::fileserv::file_and_error_handler;
    use todo_leptos_supabase::supabase::{AuthSession, SupabaseBackend};
    use todo_leptos_supabase::{AppState, PrefersDark};
    use tower_sessions_moka_store::MokaStore;

    tracing_subscriber::fmt::init();

    // Setting get_configuration(None) means we'll be using cargo-leptos's env values
    // For deployment these variables are:
    // <https://github.com/leptos-rs/start-axum#executing-a-server-on-a-remote-machine-without-the-toolchain>
    // Alternately a file can be specified such as Some("Cargo.toml")
    // The file would need to be included with the executable when moved to deployment
    let conf = get_configuration(None).await.unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    let session_store_cache = MokaStore::new(Some(100));
    let supabase = SupabaseBackend::new(session_store_cache);

    let auth_layer = AuthManagerLayerBuilder::new(
        supabase.as_auth_backend(),
        SessionManagerLayer::new(supabase.as_session_store()).with_name("auth"),
    )
    .build();

    async fn server_fn_handler(
        State(AppState {
            leptos_options,
            supabase,
        }): State<AppState>,
        Extension(auth_session): Extension<AuthSession>,
        req: Request<Body>,
    ) -> impl IntoResponse {
        handle_server_fns_with_context(
            move || {
                provide_context(auth_session.clone());
                provide_context(Arc::clone(&supabase));
                provide_context(leptos_options.clone());
            },
            req,
        )
        .await
    }

    async fn leptos_routes_handler(
        State(app_state): State<AppState>,
        Extension(auth_session): Extension<AuthSession>,
        uri: axum::extract::OriginalUri,
        req: Request<Body>,
    ) -> Response {
        let prefers_dark = PrefersDark(
            CookieJar::from_headers(req.headers())
                .get("dark_mode")
                .is_some(),
        );

        let handler = leptos_axum::render_app_to_stream_with_context(
            app_state.leptos_options,
            move || {
                provide_context(auth_session.clone());
                provide_context(uri.clone());
                provide_context(prefers_dark.clone());
            },
            App,
        );
        handler(req).await.into_response()
    }

    let app_state = AppState {
        leptos_options,
        supabase,
    };

    // build our application with a route
    let app = Router::new()
        .route(
            "/api/*server_fn",
            get(server_fn_handler).post(server_fn_handler),
        )
        .route(
            "/auth/*server_fn",
            get(server_fn_handler).post(server_fn_handler),
        )
        .route(
            "/todo/*server_fn",
            get(server_fn_handler).post(server_fn_handler),
        )
        .leptos_routes_with_handler(routes, leptos_routes_handler)
        .layer(auth_layer)
        .fallback(file_and_error_handler)
        .with_state(app_state);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    logging::log!("listening on http://{}", &addr);
    axum::serve(listener, app.into_make_service())
        .await
        .unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {
    // no client-side main function
    // unless we want this to work with e.g., Trunk for a purely client-side app
    // see lib.rs for hydration function instead
}
