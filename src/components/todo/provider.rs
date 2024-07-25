use std::collections::HashMap;

use leptos::*;

use super::{Task, TaskSchema, Tasks};

#[island]
pub fn TasksProvider(children: Children) -> impl IntoView {
    let fetch_tasks_action = create_server_action::<TodoFetch>();
    let tasks_resource =
        create_local_resource(move || fetch_tasks_action.value().get(), |_| todo_fetch());

    provide_context(Tasks {
        signal: RwSignal::new(HashMap::new()),
    });
    provide_context(tasks_resource);

    let fetched = move || {
        if let Some(Ok(t)) = tasks_resource() {
            let tasks = t
                .into_iter()
                .map(|(id, task)| {
                    let task_signal = RwSignal::new(Task {
                        title: task.title.unwrap_or_default(),
                        description: task.description.unwrap_or_default(),
                        completed: task.completed.unwrap_or_default(),
                    });
                    (id, task_signal)
                })
                .collect();

            let ctx = expect_context::<Tasks>();
            (ctx.signal).set(tasks);
        };
    };

    view! {
        <Transition fallback=|| ()>{fetched()}</Transition>

        {children()}
    }
}

#[cfg(feature = "ssr")]
#[path = ""]
mod ssr {
    pub use crate::compose_from_fn;
    pub use crate::middlewares::require_login;
}
use server_fn::codec::GetUrl;
#[cfg(feature = "ssr")]
pub use ssr::*;

#[server(prefix = "/todo", endpoint = "fetch", input = GetUrl)]
#[middleware(compose_from_fn!(require_login))]
async fn todo_fetch(/* filter? */) -> Result<HashMap<u32, TaskSchema>, ServerFnError> {
    use crate::supabase::{AuthSession, Supabase, SupabaseError};
    use axum::Extension;

    let Extension(auth_session) = leptos_axum::extract::<Extension<AuthSession>>().await?;
    let supabase = expect_context::<Supabase>();
    let res_options = expect_context::<leptos_axum::ResponseOptions>();

    let (user_id, user_token) = {
        let user = auth_session.user.unwrap();
        (user.identity.user_id, user.identity.auth_token)
    };

    // TODO: Pass the built query to a function that caches the response
    let query_response = supabase
        .client
        .query()
        .from("tasks")
        .select("id,title,description,completed")
        .eq("author_id", user_id.clone())
        .auth(user_token)
        .execute()
        .await;

    match supabase_rust::parse_response::<TaskSchema>(query_response).await {
        Ok(tasks) => {
            let map: HashMap<_, _> = if !tasks.is_empty() {
                tasks.into_iter().map(|t| (t.id.unwrap(), t)).collect()
            } else {
                HashMap::new()
            };
            Ok(map)
        }
        Err(e) => {
            let (code, err) = SupabaseError(e).into();
            res_options.set_status(code);
            Err(err)
        }
    }
}
