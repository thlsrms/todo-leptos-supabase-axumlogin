use leptos::*;

use crate::components::auth::AuthProvider;
use crate::components::todo::{NewTaskForm, TasksProvider, TasksTable};

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
      <AuthProvider>
        <hr class="uk-divider-small"/>
        <section class="uk-flex uk-flex-column uk-flex-middle ">
          <TasksProvider>
            <NewTaskForm/>
            <hr class="uk-divider-small"/>
            <TasksTable/>
          </TasksProvider>
        </section>
      </AuthProvider>
    }
}
