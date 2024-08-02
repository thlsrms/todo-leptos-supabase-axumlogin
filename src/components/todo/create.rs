use leptos::*;
use leptos_router::{ActionForm, FromFormData};

use super::{Task, TaskSchema, Tasks};

#[island]
pub fn NewTaskForm() -> impl IntoView {
    let tasks = expect_context::<Tasks>();
    let new_task_action = create_server_action::<TodoCreate>();
    let title = RwSignal::new("".to_string());
    let description = RwSignal::new("".to_string());

    let on_submit = move |ev: ev::SubmitEvent| {
        ev.prevent_default();
        match TodoCreate::from_event(&ev) {
            Ok(new_task) => {
                new_task_action.dispatch(new_task);
                title.set("".to_string());
                description.set("".to_string());
            }
            Err(e) => {
                logging::error!("{e}");
            }
        }
    };

    view! {
      <Transition fallback=|| ()>
        {move || {
            if let Some(Ok(t)) = new_task_action.value().get() {
                let task = Task {
                    title: t.title.unwrap(),
                    description: t.description.unwrap(),
                    completed: t.completed.unwrap(),
                };
                tasks
                    .signal
                    .update(|m| {
                        m.insert(t.id.unwrap(), RwSignal::new(task));
                    });
            }
        }}

      </Transition>
      <ActionForm action=new_task_action on:submit=on_submit>
        <NewTaskFormFields title=title description=description/>
      </ActionForm>
    }
}

#[component]
pub fn NewTaskFormFields(title: RwSignal<String>, description: RwSignal<String>) -> impl IntoView {
    view! {
      <div class="uk-grid-row-collapse uk-flex-middle" uk-grid>
        <div class="uk-width-1-4">
          <div class="uk-inline">
            <span class="uk-form-icon" uk-icon="triangle-right"></span>
            <input
              name="title"
              type="text"
              placeholder="New Task Title"
              aria-label="New Task Title"
              maxlength="60"
              required
              class="uk-input uk-form-blank"
              prop:value=title
            />
          </div>
        </div>
        <div class="uk-width-large@s">
          <div class="uk-inline uk-width-large">
            <span class="uk-form-icon" uk-icon="triangle-right"></span>
            <input
              name="description"
              type="text"
              placeholder="New Task Description"
              aria-label="New Task Description"
              maxlength="300"
              class="uk-input uk-form-blank"
              prop:value=description
            />
          </div>
        </div>
        <div class="uk-width-auto@s">
          <button
            type="submit"
            class="uk-button uk-button-small uk-text-success uk-text-bolder"
            uk-icon="plus-circle"
          ></button>
        </div>
      </div>
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

#[server(prefix = "/todo", endpoint = "create")]
#[middleware(compose_from_fn!(require_login))]
async fn todo_create(title: String, description: String) -> Result<TaskSchema, ServerFnError> {
    use super::TaskSchema;
    use crate::supabase::{AuthSession, Supabase};
    use axum::Extension;

    let Extension(auth_session) = leptos_axum::extract::<Extension<AuthSession>>().await?;
    let supabase = expect_context::<Supabase>();

    let (user_id, user_token) = {
        let user = auth_session.user.unwrap();
        (user.identity.user_id, user.identity.auth_token)
    };

    let task = serde_json::to_string(&TaskSchema {
        title: Some(title),
        description: Some(description),
        author_id: Some(user_id.clone()),
        ..Default::default()
    })
    .unwrap();

    // TODO: Pass the built query to a function that caches the response
    let query_response = supabase
        .client
        .query()
        .from("tasks")
        .insert(task)
        .auth(user_token)
        .execute()
        .await;

    let new_task = supabase_rust::parse_response::<TaskSchema>(query_response)
        .await
        .map_err(crate::supabase::map_err)?;
    Ok(new_task[0].clone())
}
