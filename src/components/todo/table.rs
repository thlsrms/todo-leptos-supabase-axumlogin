use super::{Task, TaskCheckbox, TaskDelete, TaskDescription, TaskEdit, TaskTitle, Tasks};
use leptos::*;

#[island]
pub fn TasksTable() -> impl IntoView {
    let tasks = expect_context::<Tasks>();

    view! {
      <div class="uk-width-2xlarge@s uk-width-1-1@l uk-text-left uk-overflow-auto">
        <table class="uk-table  uk-table-middle uk-table-divider uk-table-justify">
          <caption>"Your Tasks"</caption>
          <thead>
            <tr>
              <th class="uk-width-small"></th>
              <th class="uk-width-medium">Title</th>
              <th class="uk-width-xlarge">Description</th>
              <th class="uk-width-small"></th>
            </tr>
          </thead>
          <tbody>
            <For
              each=tasks.signal
              key=|(id, _)| *id
              children=move |(id, task): (u32, RwSignal<Task>)| {
                  view! {
                    <tr>
                      <td>
                        <TaskEdit task=task id=id/>
                      </td>
                      <td class="uk-text-break uk-height-max-small uk-overflow-auto">
                        <TaskTitle task=task/>
                      </td>
                      <td class="uk-text-break uk-height-max-small uk-overflow-auto">
                        <TaskDescription task=task/>
                      </td>
                      <td>
                        <div class="uk-flex uk-flex-middle uk-flex-nowrap">
                          <TaskCheckbox task=task id=id/>
                          <TaskDelete id=id/>

                        </div>
                      </td>
                    </tr>
                  }
              }
            />

          </tbody>
        </table>
      </div>
    }
}
