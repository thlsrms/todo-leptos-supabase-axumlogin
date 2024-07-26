//! Split each table column into its own component
//! The <leptos-island> tag would wrap a row breaking the styling

#![cfg_attr(feature = "ssr", allow(unused))]

use leptos::*;

use super::{Task, TaskSchema, Tasks};

#[island]
pub fn TaskEdit(task: RwSignal<Task>, id: u32) -> impl IntoView {
    let tasks = expect_context::<Tasks>();
    let update_task_action = create_server_action::<TodoUpdate>();
    let title_input = RwSignal::new(String::default());
    let description_input = RwSignal::new(String::default());
    let prefers_dark = RwSignal::new(false);

    prefers_dark.set(crate::PrefersDark::check());

    let edit_task = move |_| {
        update_task_action.dispatch(TodoUpdate {
            id,
            updated_task: TaskSchema {
                title: Some(title_input()),
                description: Some(description_input()),
                ..Default::default()
            },
        });

        task.update(|t| {
            t.title = title_input();
            t.description = description_input();
        });
        tasks.signal.update(|_| {});
    };

    let reset_inputs = move |_| {
        title_input.set(task().title);
        description_input.set(task().description);
    };

    view! {
        {move || {
            title_input.set(task().title);
            description_input.set(task().description);
        }}

        <div id=&format!("edit-task_{id}") class="uk-flex-top" uk-modal>
            <div class=move || format!("uk-modal-dialog uk-modal-body uk-margin-auto-vertical uk-background-{0} uk-{1} bg-toggle",
                if prefers_dark() {"secondary"} else {"default"},  if prefers_dark() {"light"} else {"dark"} )
            >
                <input
                    name="title"
                    type="text"
                    placeholder="Task Title"
                    aria-label="Task Title"
                    maxlength="60"
                    required
                    class="uk-modal-title uk-input "
                    on:input=move |ev| title_input.set(event_target_value(&ev))
                    prop:value=title_input
                />

                <br/>
                <hr/>

                <textarea
                    name="description"
                    rows="3"
                    placeholder="Task Description"
                    aria-label="Task Description"
                    maxlength="300"
                    class="uk-textarea"
                    on:input=move |ev| description_input.set(event_target_value(&ev))
                    prop:value=description_input
                ></textarea>
                <p class="uk-text-right">
                    <button
                        class="uk-button uk-button-default uk-modal-close"
                        type="button"
                        on:click=reset_inputs
                    >
                        Cancel
                    </button>
                    <button
                        class="uk-button uk-button-primary uk-modal-close"
                        type="button"
                        on:click=edit_task
                    >
                        Save
                    </button>
                </p>
            </div>
        </div>

        <button
            type="button"
            class="uk-button uk-button-small"
            uk-icon="icon: pencil"
            uk-toggle=&format!("target: #edit-task_{id}")
        ></button>
    }
}

#[island]
pub fn TaskTitle(task: RwSignal<Task>) -> impl IntoView {
    view! {
        {move || {
            if task().completed {
                view! {
                    <p class="uk-text-muted">
                        <s>{move || task().title}</s>
                    </p>
                }
            } else {
                view! { <p class="uk-text-emphasis">{move || task().title}</p> }
            }
        }}
    }
}

#[island]
pub fn TaskDescription(task: RwSignal<Task>) -> impl IntoView {
    view! {
        {move || {
            if task().completed {
                view! {
                    <p class="uk-text-muted">
                        <s>{move || task().description}</s>
                    </p>
                }
            } else {
                view! { <p>{move || task().description}</p> }
            }
        }}
    }
}

#[island]
pub fn TaskCheckbox(task: RwSignal<Task>, id: u32) -> impl IntoView {
    let tasks = expect_context::<Tasks>();
    let update_task_action = create_server_action::<TodoUpdate>();
    let completed = RwSignal::new(false);

    let update_task = move |ev| {
        completed.set(event_target_checked(&ev));
        update_task_action.dispatch(TodoUpdate {
            id,
            updated_task: TaskSchema {
                completed: Some(completed()),
                ..Default::default()
            },
        });

        task.update(|t| {
            t.completed = completed();
        });
        tasks.signal.update(|_| {});
    };

    view! {
        {move || completed.set(task().completed)}

        <div class="uk-flex-item-auto">
            <input
                type="checkbox"
                prop:checked=completed
                on:change=update_task
                class="uk-checkbox"
            />
        </div>
    }
}

#[island]
pub fn TaskDelete(id: u32) -> impl IntoView {
    let tasks = expect_context::<Tasks>();
    let prefers_dark = RwSignal::new(false);

    prefers_dark.set(crate::PrefersDark::check());

    let delete_task = move |_| {
        create_server_action::<TodoDelete>().dispatch(TodoDelete { id });

        tasks.signal.update(|m| {
            m.remove(&id);
        });
    };

    view! {
        <div id=&format!("confirm-delete_{id}") class="uk-flex-top" uk-modal>
            <div class=move || format!("uk-modal-dialog uk-modal-body uk-margin-auto-vertical uk-background-{0} uk-{1} bg-toggle",
            if prefers_dark() {"secondary"} else {"default"},  if prefers_dark() {"light"} else {"dark"} )
            >
                <h4 class="uk-modal-title uk-text-center">Delete Task?</h4>
                <p class="uk-text-center">
                    <button class="uk-button uk-button-default uk-modal-close" type="button">
                        Cancel
                    </button>
                    <button
                        class="uk-button uk-button-danger uk-modal-close"
                        type="button"
                        on:click=delete_task
                    >
                        Delete
                    </button>
                </p>
            </div>
        </div>

        <button
            type="button"
            uk-icon="trash"
            uk-toggle=&format!("target: #confirm-delete_{id}")
            class="uk-button uk-button-small uk-text-danger"
        ></button>

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

#[server(prefix = "/todo", endpoint = "update")]
#[middleware(compose_from_fn!(require_login))]
async fn todo_update(id: u32, updated_task: TaskSchema) -> Result<(), ServerFnError> {
    use super::TaskSchema;
    use crate::supabase::{AuthSession, Supabase, SupabaseError};
    use axum::Extension;

    let Extension(auth_session) = leptos_axum::extract::<Extension<AuthSession>>().await?;
    let supabase = expect_context::<Supabase>();
    let res_options = expect_context::<leptos_axum::ResponseOptions>();

    let user_token = {
        let user = auth_session.user.unwrap();
        user.identity.auth_token
    };

    // TODO: Pass the built query to a function that caches the response
    let query_response = supabase
        .client
        .query()
        .from("tasks")
        .update(serde_json::to_string(&updated_task).unwrap())
        .eq("id", id.to_string())
        .auth(user_token)
        .execute()
        .await;

    match supabase_rust::parse_response::<TaskSchema>(query_response).await {
        Ok(_) => Ok(()),
        Err(e) => {
            let (code, err) = SupabaseError(e).into();
            res_options.set_status(code);
            Err(err)
        }
    }
}

#[server(prefix = "/todo", endpoint = "delete")]
#[middleware(compose_from_fn!(require_login))]
async fn todo_delete(id: u32) -> Result<(), ServerFnError> {
    use super::TaskSchema;
    use crate::supabase::{AuthSession, Supabase, SupabaseError};
    use axum::Extension;

    let Extension(auth_session) = leptos_axum::extract::<Extension<AuthSession>>().await?;
    let supabase = expect_context::<Supabase>();
    let res_options = expect_context::<leptos_axum::ResponseOptions>();

    let user_token = {
        let user = auth_session.user.unwrap();
        user.identity.auth_token
    };

    // TODO: Pass the built query to a function that caches the response
    let query_response = supabase
        .client
        .query()
        .from("tasks")
        .delete()
        .eq("id", id.to_string())
        .auth(user_token)
        .execute()
        .await;

    match supabase_rust::parse_response::<TaskSchema>(query_response).await {
        Ok(_) => Ok(()),
        Err(e) => {
            let (code, err) = SupabaseError(e).into();
            res_options.set_status(code);
            Err(err)
        }
    }
}
