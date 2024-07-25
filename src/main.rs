#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use std::sync::Arc;

    use axum::Router;
    use axum_login::tower_sessions::SessionManagerLayer;
    use axum_login::AuthManagerLayerBuilder;
    use leptos::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use todo_leptos_supabase::app::*;
    use todo_leptos_supabase::fileserv::file_and_error_handler;
    use todo_leptos_supabase::supabase::SupabaseBackend;
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

    // build our application with a route
    let app = Router::new()
        .leptos_routes_with_context(
            &leptos_options,
            routes,
            move || provide_context(Arc::clone(&supabase)),
            App,
        )
        .layer(auth_layer)
        .fallback(file_and_error_handler)
        .with_state(leptos_options);

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
