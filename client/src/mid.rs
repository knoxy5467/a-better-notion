//! Middleware Logic
#![allow(unused)]
// for my sanity developing (TODO: remove this later)
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
#[derive(Debug, Default, Clone, PartialEq, Eq)]
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
    pub properties: Vec<TaskProp>,
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
    pub fn create_with_just_name(name: String) -> Task {
        Task {
            id: -1,           //-1 is a placeholder for a task that is not in the database
            completed: false, // all tasks start with completed as false
            name,
            ..Default::default()
        }
    }
}

/// Middleware stored View
#[derive(Debug, Default, Clone, Eq, PartialEq)]
pub struct View {
    /// Database ID of the view
    pub db_id: ViewID,
    /// Name of the view
    pub name: String,
    /// Filter for view
    pub filter: Filter,
    /// Properties shown in view
    pub props: Vec<String>,
    /// Tasks that are apart of the view, calculated on the backend via calls to /filterids
    pub tasks: Option<Vec<TaskID>>,
}

/// Middleware State structure.
#[derive(Debug)]
pub struct State {
    /// maps between database ID and middleware ID for task
    /// If task is only stored locally, may not contain entry for task key
    /// TaskIDs here must have corresponding Task in hashmap
    task_map: HashMap<TaskID, Task>,
    /// store prop names with unique keys
    prop_map: HashMap<(TaskID, String), TaskPropVariant>,
    /// properties stored in the middleware can be uniquely identified by the task they are stored upon and the string of the property
    /// connected url
    url: String,
    /// reqwest client
    client: reqwest::Client,
    /// Connection status
    status: bool,
    request_count: i32,
    ///map of views
    view_map: HashMap<ViewID, View>,
}
impl State {
    fn increment_and_get_request_count(&mut self) -> i32 {
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
    pub async fn task_rm(&mut self, key: TaskID) {
        let delete_task_request = DeleteTaskRequest {
            task_id: key,
            req_id: self.increment_and_get_request_count(),
        };
        let url = self.url.clone();
        let deleted_task_response = self
            .client
            .delete(url + "/task")
            .json(&delete_task_request)
            .send()
            .await
            .unwrap();
        let deleted_task_id = deleted_task_response
            .json::<DeleteTaskResponse>()
            .await
            .unwrap();
        if deleted_task_id != delete_task_request.req_id {
            panic!(
                "Task ID mismatch, we just deleted the wrong task! deleted_id: {:?}, key: {:?}",
                deleted_task_id, key
            );
        }
        self.task_map.remove(&key);
        for view in self.view_map.values_mut() {
            if let Some(tasks) = view.tasks.as_mut() {
                tasks.retain(|&x| x != key);
            }
        }
    }
    /// get a task by its id
    pub fn get_task(&self, task_id: TaskID) -> Option<&Task> {
        if (self.task_map.contains_key(&task_id)) {
            return self.task_map.get(&task_id);
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
        let url = &self.url;
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
            self.task_map.insert(task.task_id, new_task);
        }
    }
    pub fn add_view(&mut self, view: View) {
        self.view_map.insert(view.db_id, view);
    }
    /// get a list of task IDs associated with a viewID
    pub fn view_tasks(&self, view_id: ViewID) -> Option<Vec<TaskID>> {
        return self.view_map.get(&view_id).unwrap().tasks.clone();
    }
    /// modify a task using a given function
    pub async fn modify_task(&mut self, task_id: TaskID, edit_fn: impl FnOnce(&mut Task)) {
        let before_task = self.task_map.get(&task_id).unwrap().clone();
        if let Some(task) = self.task_map.get_mut(&task_id) {
            edit_fn(task)
        }
        let req_id = self.increment_and_get_request_count();
        let after_task = self.task_map.get_mut(&task_id).unwrap();
        if before_task != *after_task {
            let url = &self.url;
            let before_deps_set = before_task
                .dependencies
                .iter()
                .collect::<std::collections::HashSet<_>>();
            let after_deps_set = after_task
                .dependencies
                .iter()
                .collect::<std::collections::HashSet<_>>();
            let deps_to_add = after_deps_set
                .difference(&before_deps_set)
                .cloned()
                .map(|dep| dep.clone())
                .collect::<Vec<_>>();
            let deps_to_remove = before_deps_set
                .difference(&after_deps_set)
                .cloned()
                .map(|dep| dep.clone())
                .collect::<Vec<_>>();
            let before_props_set = before_task
                .properties
                .iter()
                .collect::<std::collections::HashSet<_>>();
            let after_props_set = after_task
                .properties
                .iter()
                .collect::<std::collections::HashSet<_>>();
            let props_to_add = after_props_set
                .difference(&before_props_set)
                .cloned()
                .map(|prop| prop.clone())
                .collect::<Vec<_>>();
            let props_to_remove = before_props_set
                .difference(&after_props_set)
                .cloned()
                .map(|prop| prop.clone())
                .collect::<Vec<_>>();
            let before_scripts_set = before_task
                .scripts
                .iter()
                .collect::<std::collections::HashSet<_>>();
            let after_scripts_set = after_task
                .scripts
                .iter()
                .collect::<std::collections::HashSet<_>>();
            let scripts_to_add = after_scripts_set
                .difference(&before_scripts_set)
                .cloned()
                .map(|script| script.clone())
                .collect::<Vec<_>>();
            let scripts_to_remove = before_scripts_set
                .difference(&after_scripts_set)
                .cloned()
                .map(|script| script.clone())
                .collect::<Vec<_>>();
            let update_task_request = UpdateTaskRequest {
                task_id: task_id,
                checked: Some(after_task.completed.clone().to_owned()),
                name: Some(after_task.name.clone().to_owned()),
                req_id: req_id,
                props_to_add: props_to_add,
                props_to_remove: props_to_remove,
                deps_to_add: deps_to_add,
                deps_to_remove: deps_to_remove,
                scripts_to_add: scripts_to_add,
                scripts_to_remove: scripts_to_remove,
            };
            let response = self
                .client
                .put(format!("{url}/task"))
                .json(&update_task_request)
                .send()
                .await
                .unwrap()
                .json::<UpdateTaskResponse>()
                .await;
            match response {
                Ok(response) => {
                    if response.task_id != task_id {
                        panic!("Task ID mismatch, we just updated the wrong task! updated_id: {:?}, key: {:?}", response.task_id, task_id);
                    }
                }
                Err(e) => {
                    self.task_map.insert(task_id, before_task); //maybe they tried to do something funky and we couldnt update the dependencies
                }
            }
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
    pub async fn create_task(
        &mut self,
        task_to_be_created: Task,
    ) -> Result<CreateTaskResponse, reqwest::Error> {
        let create_task_request = CreateTaskRequest {
            name: task_to_be_created.name,
            completed: task_to_be_created.completed,
            properties: vec![],
            dependencies: task_to_be_created.dependencies,
            req_id: self.increment_and_get_request_count(),
        };
        let created_task_response: Result<CreateTaskResponse, reqwest::Error> = self
            .client
            .post(format!("{}/task", self.url))
            .json(&create_task_request)
            .send()
            .await
            .unwrap()
            .json::<CreateTaskResponse>()
            .await;
        let created_task_id = created_task_response.as_ref().unwrap().task_id;
        let mut created_task = Task::new(
            created_task_id,
            create_task_request.name,
            create_task_request.completed,
        );
        created_task.dependencies = create_task_request.dependencies;
        self.task_map.insert(created_task.id, created_task);
        return created_task_response;
    }
}
pub async fn init(url: &str) -> color_eyre::Result<State> {
    let mut state = State::new(url.to_string());
    state.get_beginning_tasks().await;
    let default_view = View {
        db_id: -1,
        name: "Default".to_string(),
        filter: Filter::None,
        props: vec![],
        tasks: Some(state.task_map.keys().cloned().collect()),
    };
    state.add_view(default_view);
    Ok(state)
}
