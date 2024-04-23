//! Middleware Logic
#![allow(unused)] // for my sanity developing (TODO: remove this later)
use color_eyre::eyre::{Context, ContextCompat};
use common::{
    backend::{
        CreateTaskRequest, CreateTaskResponse, DeleteTaskRequest, DeleteTaskResponse,
        FilterRequest, FilterResponse, ReadTaskShortRequest, ReadTaskShortResponse,
        ReadTasksShortRequest, ReadTasksShortResponse, UpdateTaskRequest, UpdateTaskResponse,
    },
    *,
};
use futures::{
    channel::mpsc::{self, Receiver, Sender},
    SinkExt, Stream, StreamExt,
};
use reqwest::Response;
use reqwest_middleware::{ClientBuilder, ClientWithMiddleware, RequestBuilder};
use reqwest_tracing::{SpanBackendWithUrl, TracingMiddleware};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use slotmap::{new_key_type, KeyData, SlotMap};
use std::{collections::HashMap, fmt};
use thiserror::Error;
use tokio::task::JoinHandle;

/// All data associated with tasks, except for properties
#[derive(Debug, Default, Clone)]
pub struct Task {
    /// if it is stored in the database, it will have a unique task_id.
    pub id: TaskID,
    /// Short name of the task (description is a property)
    pub name: String,
    /// Whether the task is completed or not
    pub completed: bool,
    /// Dependencies of this task
    pub dependencies: Vec<TaskID>,
    /// Associated scripts
    pub scripts: Vec<ScriptID>,
    pub view_map: HashMap<u64, View>,
    /// latest should be set to true if this value matches server (if false and needed, it should be fetched and updated as soon as possible)
    is_syncronized: bool,
    /// if task is pending deletion request
    pending_deletion: bool,
}
impl Task {
    pub fn new(task_id: TaskID, name: String, completed: bool) -> Task {
        Task {
            id: task_id,
            name,
            completed,
            ..Default::default()
        }
    }
}

/// Middleware stored View
#[derive(Debug, Default, Copy, Clone)]
pub struct View {
    /// Database ID of the view
    pub db_id: ViewID,
    /// Name of the view
    pub name: String,
    /// Filter for view
    pub filter: Filter,
    /// Properties shown in view
    pub props: Vec<PropNameKey>,
    /// Tasks that are apart of the view, calculated on the backend via calls to /filterids
    pub tasks: Option<Vec<TaskKey>>,
}

/// Middleware State structure.
#[derive(Debug)]
pub struct State {
    /// maps between database ID and middleware ID for task
    /// If task is only stored locally, may not contain entry for task key
    /// TaskIDs here must have corresponding Task in hashmap
    task_map: HashMap<TaskID, TaskKey>,
    /// store prop names with unique keys
    prop_map: HashMap<(TaskID, PropNameKey), TaskPropVariant>,
    /// properties stored in the middleware can be uniquely identified by the task they are stored upon and the string of the property
    /// connected url
    url: String,
    /// reqwest client
    client: reqwest::Client,
    /// Connection status
    status: bool,
    request_count: u64,
    ///map of views
    view_map: HashMap<ViewID, View>,
}
impl State {
    fn increment_and_get_request_count(&mut self) -> u64 {
        self.request_count += 1;
        self.request_count
    }
    fn new(url: String) -> Self {
        Self {
            task_map: HashMap::new(),
            prop_map: HashMap::new(),
            url: url,
            status: false,
            client: reqwest::Client::new(),
            request_count: 0,
            view_map: HashMap::new(),
        }
    }
    pub async fn task_rm(&mut self, key: TaskId) {
        let delete_task_request = DeleteTaskRequest {
            task_id: key,
            req_id: self.increment_and_get_request_count(),
        };
        let response = self
            .client
            .delete(self.url + "/task/")
            .json(&delete_task_request)
            .send()
            .await
            .unwrap()
            .json::<DeleteTaskResponse>()
            .await
            .unwrap();
    }
    /// get a task by its id
    pub fn get_task(&self, task_id: TaskID) -> Option<&Task> {
        if (self.task_map.contains_key(&task_id)) {
            return Some(self.task_map.get(&task_id));
        } else {
            return None; // TODO 23APR2024: change this to try to get it from the server
        }
    }
    /// populate task map with tasks from the server, generally should be used for init
    pub async fn get_beginning_tasks(&mut self) {
        // request all tasks using a "None" filter
        let filter_request = FilterRequest {
            filter: Filter::None,
        };
        let url = self.url;
        let task_ids: Vec<TaskID> = self
            .client
            .get(format!("{url}/filter"))
            .json(&filter_request)
            .send()
            .await
            .unwrap()
            .json::<FilterResponse>()
            .await
            .unwrap();
        let task_requests = task_ids
            .into_iter()
            .map(|task_id| ReadTaskShortRequest { task_id, req_id: 0 })
            .collect::<ReadTasksShortRequest>();
        let tasks = self
            .client
            .get(format!("{url}/tasks"))
            .json(&task_requests)
            .send()
            .await
            .unwrap()
            .json::<ReadTasksShortResponse>()
            .await
            .unwrap();
        for task_result in tasks {
            let task = task_result.unwrap();
            let new_task = Task::new(task.task_id, task.name, task.completed);
            self.task_map.insert(task.id, task);
        }
    }
    pub fn add_view(&mut self, view: View) {
        self.view_map.insert(view.db_id, view);
    }
    /// get a list of task IDs associated with a viewID
    pub fn view_tasks(&self, view_id: ViewID) -> Option<&[TaskId]> {
        return self
            .view_map
            .get(&view_id)
            .map(|view| view.tasks.as_slice());
    }
    /// modify a task using a given function
    pub fn modify_task(&mut self, task_id: TaskID, edit_fn: impl FnOnce(&mut Task)) {
        if let Some(task) = self.task_mapk.get_mut(task_id) {
            edit_fn(task)
        }
    }
    ///modify view by passing in a function and running the given function
    pub fn modify_view(&mut self, view_id: ViewID, edit_fn: impl FnOnce(&mut View)) -> Option<()> {
        edit_fn(self.view_map.get_mut(&view_id)?);
        None
    }
    pub fn get_default_view(&self) -> Option<&View> {
        return self.view_map.get(&-1); //the default view should always be with id -1
    }
    /// creates a task in the server, adds that task to the state task list and returns the ID in a result, or an error if the server could not perform the action.
    pub async fn create_task(&self, task_to_be_created: Task) -> Result<CreateTaskResponse, Error> {
        let create_task_request = CreateTaskRequest {
            name: task_to_be_created.name,
            completed: task_to_be_created.completed,
            properties: vec![],
            dependencies: task_to_be_created.dependencies,
            req_id: self.increment_and_get_request_count(),
        };
        let created_task_response = self
            .client
            .post(format!("{}/task", self.url))
            .json(&create_task_request)
            .send()
            .await
            .unwrap()
            .json::<CreateTaskResponse>()
            .await
            .unwrap();
        return created_task_response.task_id;
    }
}
pub fn init(url: &str) -> color_eyre::Result<State> {
    let mut state = State::new(url.to_string());
    state.get_beginning_tasks();
    let default_view = View {
        db_id: -1,
        name: "Default".to_string(),
        filter: Filter::None,
        props: vec![],
        tasks: state.task_map.keys().cloned().collect(),
    };
    state.add_view(default_view);
    Ok(state)
}
