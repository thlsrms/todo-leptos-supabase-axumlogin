mod create;
mod provider;
mod table;
mod tablerow;

pub use create::NewTaskForm;
pub use provider::TasksProvider;
pub use table::TasksTable;

use std::collections::HashMap;

use leptos::RwSignal;
use tablerow::{TaskCheckbox, TaskDelete, TaskDescription, TaskEdit, TaskTitle};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Tasks {
    signal: RwSignal<HashMap<u32, RwSignal<Task>>>,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Default, Clone, PartialEq)]
pub struct Task {
    pub title: String,
    pub completed: bool,
    pub description: String,
}

#[derive(serde::Deserialize, serde::Serialize, Debug, Default, Clone, PartialEq)]
pub struct TaskSchema {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed: Option<bool>,
    pub description: Option<String>,
}
