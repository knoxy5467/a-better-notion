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
use std::{
    collections::{HashMap, HashSet},
    fmt,
};
use thiserror::Error;
use tokio::task::JoinHandle;

new_key_type! { pub struct PropKey; }
new_key_type! { pub struct TaskKey; }

/// All data associated with tasks, except for properties
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Task {
    /// Short name of the task (description is a property)
    pub name: String,
    /// Whether the task is completed or not
    pub completed: bool,
    /// Dependencies of this task
    pub dependencies: Vec<TaskKey>,
    /// Associated scripts
    pub scripts: Vec<ScriptID>,
    /// if it is stored in the database, it will have a unique task_id.
    pub db_id: Option<TaskID>,
    /// latest should be set to true if this value matches server (if false and needed, it should be fetched and updated as soon as possible)
    pub is_syncronized: bool,
    /// if task is pending deletion request
    pub pending_deletion: bool,
}
impl Task {
    pub fn new(name: String, completed: bool) -> Task {
        Task {
            name,
            completed,
            ..Default::default()
        }
    }
}

/// Middleware stored View
#[derive(Debug, Default, PartialEq)]
pub struct View {
    /// Name of the view
    pub name: String,
    /// Filter for view
    pub filter: Filter,
    /// Properties shown in view
    pub props: Vec<PropNameKey>,
    /// Tasks that are apart of the view, calculated on the backend via calls to /filterids
    pub tasks: Option<Vec<TaskKey>>,
    /// Computed task list for view
    pub db_id: Option<ViewID>,
}

impl View {
    pub fn new(name: String) -> View {
        View {
            name,
            ..Default::default()
        }
    }
}

new_key_type! { pub struct PropNameKey; }
new_key_type! { pub struct ViewKey; }

/// Middleware State structure.
#[derive(Debug)]
pub struct State {
    /// maps between database ID and middleware ID for task
    /// If task is only stored locally, may not contain entry for task key
    /// TaskIDs here must have corresponding Task in hashmap
    task_map: HashMap<TaskID, TaskKey>,
    /// stores task in dense datastructure for iteration efficiency
    tasks: SlotMap<TaskKey, Task>,

    /// store prop names with unique keys
    prop_names: SlotMap<PropNameKey, String>,
    /// lookup prop name key with string
    prop_name_map: HashMap<PropName, PropNameKey>,
    /// properties stored in the middleware can be uniquely identified by the task they are stored upon and the string of the property
    prop_map: HashMap<(TaskKey, PropNameKey), PropKey>,
    /// efficient, dense storage of all locally-stored task properties
    props: SlotMap<PropKey, TaskPropVariant>,

    /// scripts are identified by database's ScriptID
    scripts: HashMap<ScriptID, Script>,
    /// views are identified by database's ViewID
    views_map: HashMap<ViewID, ViewKey>,
    views: SlotMap<ViewKey, View>,
    /// connected url
    url: String,
    client: ClientWithMiddleware,
    /// Connection status
    status: bool,
    mid_event_sender: Sender<MidEvent>,
}
#[derive(Debug)]
/// Events to be handled by the middleware
pub enum MidEvent {
    ServerResponse(Result<Box<dyn ServerResponse>, reqwest_middleware::Error>),
    StateEvent(StateEvent), // events to be handled by the ui
}
impl State {
    // handles MidEvents, so long as MidEvent is not of variant StateEvent
    pub fn handle_mid_event(&mut self, event: MidEvent) -> color_eyre::Result<()> {
        match event {
            MidEvent::ServerResponse(Ok(resp)) => {
                if let Some(event) = resp.update_state(self)? {
                    self.mid_event_sender
                        .try_send(MidEvent::StateEvent(event))?;
                }
            }
            MidEvent::ServerResponse(Err(err)) => Err(err)?,
            MidEvent::StateEvent(_) => panic!("middleware does not handle state events"),
        }
        Ok(())
    }
}

/// Error returned if property does not exist
#[derive(Debug, Error)]
pub enum PropDataError {
    /// Task does not exist.
    #[error("task identified by unique key {0:?} does not exist")]
    Task(TaskKey),
    /// Property name does not exists.
    #[error("property name identified by unique key {0:?} does not exist")]
    PropertyName(PropNameKey),
    /// Property does not exist
    #[error("property associated with task {0:?} and prop name: {0:?} does not exist")]
    Prop(TaskKey, PropNameKey),
}

/// Anything in this enum is sent to the middleware script executor when a UI event is triggered.
enum ScriptEvent {
    /// Name of the event
    RegisteredEvent(String),
}

/// Event sent to UI via channel to notify UI that some data has changed and the render should be updated. (Note: These can and should be made more granular for better performance)
#[derive(Debug)]
pub enum StateEvent {
    /// A bunch of tasks were updated
    TasksUpdate,
    /// One or more properties were updated
    PropsUpdate,
    /// One or more views updated
    ViewsUpdate,
    /// A script was updated
    ScriptUpdate(ScriptID),
    /// The connection has either connected or disconnected.
    ServerStatus(bool),
}

// data events received from server must implement this trait to be applied to middleware
pub trait ServerResponse: fmt::Debug + Send + Sync + 'static {
    fn update_state(self: Box<Self>, state: &mut State) -> color_eyre::Result<Option<StateEvent>>;
}
impl ServerResponse for ReadTaskShortResponse {
    // note: this shouldn't error, if it does in the future, make sure to handle them in the ReadTasksShortResponse ServerResponse impl
    fn update_state(self: Box<Self>, state: &mut State) -> color_eyre::Result<Option<StateEvent>> {
        // create task from received info
        let mut task = Task {
            name: self.name,
            completed: self.completed,
            dependencies: self
                .deps
                .iter()
                .map(|tid| state.new_server_task(*tid).0)
                .collect::<Vec<TaskKey>>(),
            scripts: self.scripts,
            db_id: Some(self.task_id),
            is_syncronized: true,
            pending_deletion: false,
        };
        // create/update existing task with read task
        *state.new_server_task(self.task_id).1 = task;
        Ok(Some(StateEvent::TasksUpdate))
    }
}
// should only receive this if we already know the server has the task (i.e. sent CreateTaskResponse)
impl ServerResponse for UpdateTaskResponse {
    fn update_state(self: Box<Self>, state: &mut State) -> color_eyre::Result<Option<StateEvent>> {
        let task_key = state.task_map.get(&self.task_id).with_context(|| {
            format!(
                "cannot find locally stored task associated with DB id: {:?}",
                self.task_id
            )
        })?;
        if let Some(task) = state.tasks.get_mut(*task_key) {
            task.db_id = Some(self.task_id);
            task.is_syncronized = true;
        } else {
            panic!(
                "fatal: DB id {:?} was associated with task key {:?} but task didn't exist",
                self.task_id, task_key
            );
        }
        Ok(Some(StateEvent::TasksUpdate))
    }
}
impl ServerResponse for CreateTaskResponse {
    fn update_state(self: Box<Self>, state: &mut State) -> color_eyre::Result<Option<StateEvent>> {
        let task_key = TaskKey(slotmap::KeyData::from_ffi(self.req_id));
        let task = state.tasks.get_mut(task_key).with_context(||format!("req_id received from CreateTaskResponse does not match a local task key: {task_key:?}"))?;
        task.db_id = Some(self.task_id); // record db ID
        state.task_map.insert(self.task_id, task_key); // record in db map
        task.is_syncronized = true; // flag syncronized
        Ok(Some(StateEvent::TasksUpdate))
    }
}
impl ServerResponse for DeleteTaskResponse {
    fn update_state(self: Box<Self>, state: &mut State) -> color_eyre::Result<Option<StateEvent>> {
        // get task key from response (should have been sent with request)
        let task_key = TaskKey(slotmap::KeyData::from_ffi(*self));
        // remove task key
        let task = state.tasks.remove(task_key).with_context(||format!("req_id received from DeleteTaskResponse does not match a local task key: {task_key:?}"))?;
        if let Some(db_id) = task.db_id {
            // if we have a local db_id, remove it from the map
            state.task_map.remove(&db_id);
        }
        Ok(Some(StateEvent::TasksUpdate))
    }
}

impl ServerResponse for ReadTasksShortResponse {
    fn update_state(self: Box<Self>, state: &mut State) -> color_eyre::Result<Option<StateEvent>> {
        // no need to handle errors here, ReadTaskShortResponse's don't return errors
        for res in (*self).into_iter().flatten() {
            Box::new(res).update_state(state);
        }
        Ok(Some(StateEvent::TasksUpdate))
    }
}
impl ServerResponse for FilterResponse {
    fn update_state(self: Box<Self>, state: &mut State) -> color_eyre::Result<Option<StateEvent>> {
        // allocate server tasks
        let tasks: Vec<TaskKey> = self
            .tasks
            .into_iter()
            .map(|tid| state.new_server_task(tid).0)
            .collect::<Vec<TaskKey>>();
        // set task keys in view
        let view_key = ViewKey(KeyData::from_ffi(self.req_id));
        let view = state.views.get_mut(view_key).with_context(|| {
            format!(
                "request id corresponding to view key: {:?} sent back was invalid",
                view_key
            )
        })?; // TODO add context
        view.tasks = Some(tasks);

        // get tasks
        let view_tasks = state.view_task_keys(view_key).unwrap();
        // calculate which tasks we have and which need fetching using is_syncronized
        let tasks_to_fetch = view_tasks
            .filter_map(|tkey| {
                state
                    .tasks
                    .get(tkey)
                    .and_then(|t| if !t.is_syncronized { t.db_id } else { None })
            })
            .map(|task_id| ReadTaskShortRequest { task_id, req_id: 0 })
            .collect::<Vec<ReadTaskShortRequest>>();

        // automatically fetch needed tasks. TODO: to be smarter about this should we dynamically fetch based on UI (?)
        tracing::debug!("fetching tasks: {:?}", tasks_to_fetch);
        if !tasks_to_fetch.is_empty() {
            state.spawn_request::<ReadTasksShortRequest, ReadTasksShortResponse>(
                state.client.get(format!("{}/tasks", state.url)),
                tasks_to_fetch,
            );
        }
        Ok(Some(StateEvent::ViewsUpdate))
    }
}

impl State {
    /// Create a new state. This should be (mostly) used internally, use init_test() or init() for regular applications.
    pub fn new() -> (State, Receiver<MidEvent>) {
        let (mid_event_sender, receiver) = mpsc::channel(30);
        (
            State {
                task_map: Default::default(),
                tasks: Default::default(),
                prop_names: Default::default(),
                prop_name_map: Default::default(),
                prop_map: Default::default(),
                props: Default::default(),
                scripts: Default::default(),
                views_map: Default::default(),
                views: Default::default(),
                url: Default::default(),
                status: Default::default(),
                mid_event_sender,
                client: ClientBuilder::new(reqwest::Client::new())
                    .with(TracingMiddleware::<SpanBackendWithUrl>::new())
                    .build(),
            },
            receiver,
        )
    }
    // create new or return existing task given server TaskID
    fn new_server_task(&mut self, task_id: TaskID) -> (TaskKey, &mut Task) {
        if let Some(key) = self.task_map.get(&task_id).cloned() {
            (
                key,
                self.tasks.get_mut(key).expect(
                    "fatal: a key in the task_map should imply that tasks has the relevant key",
                ),
            )
        } else {
            let key = self.tasks.insert(Task::default());
            self.task_map.insert(task_id, key);
            let task = self.tasks.get_mut(key).unwrap();
            task.db_id = Some(task_id);
            (key, task)
        }
    }
    /// schedule task to wait for response from server and the notifies the client via mid_event_sender when received.
    /// TODO: Configure request timeouts
    #[tracing::instrument]
    fn spawn_request<Req, Res>(&mut self, req_builder: RequestBuilder, req: Req) -> JoinHandle<()>
    where
        Req: Serialize + std::fmt::Debug + Send + Sync + 'static,
        Res: ServerResponse + for<'d> Deserialize<'d>,
    {
        tracing::debug!("doing a request: {:?}", req);
        let mut sender = self.mid_event_sender.clone();
        tokio::spawn(async move {
            let resp = do_request::<Req, Res>(req_builder, req).await;
            let resp = resp.map(|e| Box::new(e) as Box<dyn ServerResponse>);
            tracing::debug!("received a response: {:?}", resp);
            sender.send(MidEvent::ServerResponse(resp)).await;
        })
    }
}

#[derive(Debug, Error, Clone)]
#[error("task: task associated with key {0:?} does not exist")]
pub struct NoTaskError(TaskKey);

#[derive(Debug, Error, Clone)]
#[error("task: task associated with key {0:?} was not syncronized with server")]
pub struct UnsyncronizedTaskError(TaskKey);

#[derive(Debug, Error, Clone)]
pub enum ModifyTaskError {
    #[error(transparent)]
    NoTask(#[from] NoTaskError),
    #[error(transparent)]
    UnsyncronizedTask(#[from] UnsyncronizedTaskError),
}

#[derive(Debug, Error, Clone)]
#[error("view: view associated with key {0:?} does not exist")]
pub struct NoViewError(ViewKey);

impl State {
    /// define a task, get a key that uniquely identifies it
    pub fn task_def(&mut self, task: Task) -> TaskKey {
        // TODO: register definition to queue so that we can sync to server
        let key = self.tasks.insert(task);
        let task = &self.tasks[key]; // safety: we just inserted key
        self.spawn_request::<CreateTaskRequest, CreateTaskResponse>(
            self.client.post(format!("{}/task", self.url)),
            CreateTaskRequest {
                name: task.name.clone(),
                completed: task.completed,
                properties: vec![],   // TODO: send props
                dependencies: vec![], // TODO: send deps
                req_id: key.0.as_ffi(),
            },
        );
        key
    }

    /// get task using a key, if it exists
    pub fn task_get(&self, key: TaskKey) -> Result<&Task, NoTaskError> {
        self.tasks.get(key).ok_or(NoTaskError(key))
    }
    /// modify a task
    pub fn task_mod(
        &mut self,
        key: TaskKey,
        edit_fn: impl FnOnce(&mut Task),
    ) -> Result<(), ModifyTaskError> {
        if let Some(task) = self.tasks.get_mut(key) {
            if task.is_syncronized {
                // get previous task state
                let bef = task.clone();
                edit_fn(task); // modify task
                               // send only difference between task before and after to server.
                let name = (bef.name != task.name).then_some(task.name.clone());
                let completed = (bef.completed != task.completed).then_some(task.completed);
                if let Some(db_id) = task.db_id {
                    self.spawn_request::<UpdateTaskRequest, UpdateTaskResponse>(
                        self.client.put(format!("{}/task", self.url)),
                        UpdateTaskRequest {
                            task_id: db_id,
                            name,
                            checked: completed,
                            props_to_add: vec![],
                            props_to_remove: vec![],
                            deps_to_add: vec![],
                            deps_to_remove: vec![],
                            scripts_to_add: vec![],
                            scripts_to_remove: vec![],
                            req_id: key.0.as_ffi(),
                        },
                    );
                }
                Ok(())
            } else {
                Err(UnsyncronizedTaskError(key).into())
            }
        } else {
            Err(NoTaskError(key).into())
        }
    }
    /// delete a task
    pub fn task_rm(&mut self, key: TaskKey) -> Result<(), NoTaskError> {
        // dbg!("removing a task!");
        if let Some(task) = self.tasks.get_mut(key) {
            if let Some(db_id) = task.db_id {
                // mark pending deletion if in database
                task.pending_deletion = true;
                self.spawn_request::<DeleteTaskRequest, DeleteTaskResponse>(
                    self.client.delete(format!("{}/task", self.url)),
                    DeleteTaskRequest {
                        task_id: db_id,
                        req_id: key.0.as_ffi(),
                    },
                );
            } else {
                // if not in database, remove immediately
                self.tasks.remove(key);
            }
        } else {
            return Err(NoTaskError(key));
        }
        Ok(())
    }
    /// define a property of a certain type on an associated task
    pub fn prop_def(
        &mut self,
        task_key: TaskKey,
        name_key: PropNameKey,
        prop: TaskPropVariant,
    ) -> Result<PropKey, PropDataError> {
        let Some(_) = self.tasks.get(task_key) else {
            return Err(PropDataError::Task(task_key));
        };
        let Some(_) = self.prop_names.get(name_key) else {
            return Err(PropDataError::PropertyName(name_key));
        };

        let prop_key = self.props.insert(prop);
        self.prop_map.insert((task_key, name_key), prop_key);
        Ok(prop_key)
    }
    /// define a property name
    pub fn prop_def_name(&mut self, name: impl Into<String>) -> PropNameKey {
        let name: String = name.into();
        let key = self.prop_names.insert(name.clone());
        self.prop_name_map.insert(name, key);
        key
    }
    /// delete a property name
    pub fn prop_rm_name(&mut self, name_key: PropNameKey) -> Result<String, PropDataError> {
        let name = self
            .prop_names
            .remove(name_key)
            .ok_or(PropDataError::PropertyName(name_key))?;
        self.prop_name_map.remove(&name);
        Ok(name)
    }
    /// get a property
    pub fn prop_get(
        &self,
        task_key: TaskKey,
        name_key: PropNameKey,
    ) -> Result<&TaskPropVariant, PropDataError> {
        let Some(_) = self.tasks.get(task_key) else {
            return Err(PropDataError::Task(task_key));
        };
        let Some(_) = self.prop_names.get(name_key) else {
            return Err(PropDataError::PropertyName(name_key));
        };

        let key = self
            .prop_map
            .get(&(task_key, name_key))
            .ok_or(PropDataError::Prop(task_key, name_key))?;
        Ok(&self.props[*key])
    }
    /// modify a property
    pub fn prop_mod(
        &mut self,
        task_key: TaskKey,
        name_key: PropNameKey,
        edit_fn: impl FnOnce(&mut TaskPropVariant),
    ) -> Result<(), PropDataError> {
        let Some(_) = self.tasks.get(task_key) else {
            return Err(PropDataError::Task(task_key));
        };
        let Some(_) = self.prop_names.get(name_key) else {
            return Err(PropDataError::PropertyName(name_key));
        };

        let key = self
            .prop_map
            .get(&(task_key, name_key))
            .ok_or(PropDataError::Prop(task_key, name_key))?;
        edit_fn(
            self.props
                .get_mut(*key)
                .ok_or(PropDataError::Prop(task_key, name_key))?,
        );
        Ok(())
    }
    /// delete a property
    pub fn prop_rm(
        &mut self,
        task_key: TaskKey,
        name_key: PropNameKey,
    ) -> Result<TaskPropVariant, PropDataError> {
        let Some(_) = self.tasks.get(task_key) else {
            return Err(PropDataError::Task(task_key));
        };
        let Some(_) = self.prop_names.get(name_key) else {
            return Err(PropDataError::PropertyName(name_key));
        };

        let key = self
            .prop_map
            .remove(&(task_key, name_key))
            .ok_or(PropDataError::Prop(task_key, name_key))?;
        self.props
            .remove(key)
            .ok_or(PropDataError::Prop(task_key, name_key))
    }
    /// define a view
    pub fn view_def(&mut self, view: View) -> ViewKey {
        // TODO: register to save updated view
        self.views.insert(view)
    }
    /// get a view
    pub fn view_get(&self, view_key: ViewKey) -> Result<&View, NoViewError> {
        self.views.get(view_key).ok_or(NoViewError(view_key))
    }
    /// get the default view
    pub fn view_get_default(&self) -> Option<ViewKey> {
        self.views.keys().next()
    }
    /// shorthdand function to get the list of tasks associated with a view (some keys may be invalid)
    pub fn view_task_keys(
        &self,
        view_key: ViewKey,
    ) -> Option<impl Iterator<Item = TaskKey> + Clone + '_> {
        self.view_get(view_key)
            .ok()
            .and_then(|v| v.tasks.as_ref())
            .map(|v| v.iter().cloned())
    }
    /// get an iterator of only valid tasks and their keys
    pub fn view_tasks(
        &self,
        view_key: ViewKey,
    ) -> Option<impl Iterator<Item = (TaskKey, &Task)> + Clone> {
        self.view_task_keys(view_key)
            .map(|tks| tks.flat_map(|key| self.task_get(key).ok().map(|t| (key, t))))
    }
    /// modify a view
    pub fn view_mod(&mut self, view_key: ViewKey, edit_fn: impl FnOnce(&mut View)) -> Option<()> {
        edit_fn(self.views.get_mut(view_key)?);
        None
    }
    /// delete a view
    pub fn view_rm(&mut self, view_key: ViewKey) {
        self.views.remove(view_key);
    }
    /// create a script
    pub fn script_create(&mut self) -> ScriptID {
        self.scripts.insert(0, Script::default());
        0
    }
    /// get a script
    pub fn script_get(&self, script_id: ScriptID) -> Option<&Script> {
        self.scripts.get(&script_id)
    }
    /// modify a script
    pub fn script_mod(&mut self, script_id: ScriptID, edit_fn: impl FnOnce(&mut Script)) {
        self.scripts.entry(script_id).and_modify(edit_fn);
    }
    /// delete a script
    pub fn script_rm(&mut self, script_id: ScriptID) {
        self.scripts.remove(&script_id);
    }

    /* pub fn register_event(&mut self, name: &str) {
        Default::default()
    }

    pub fn event_notify(&mut self, name: &str) -> bool {
        Default::default()
    } */
}

// request helper function
#[tracing::instrument]
async fn do_request<Req, Res>(
    req_builder: RequestBuilder,
    req: Req,
) -> reqwest_middleware::Result<Res>
where
    Req: Serialize + std::fmt::Debug,
    Res: for<'d> Deserialize<'d> + std::fmt::Debug,
{
    let res: Response = req_builder.json(&req).send().await?;
    let bytes = res.bytes().await?;
    tracing::debug!("received data: {bytes:?}");
    let res: Res = serde_json::from_slice(&bytes).map_err(reqwest_middleware::Error::middleware)?;
    Ok(res)
}

/// Init middleware state
/// This function is called by UI to create the Middleware state and establish a connection to the Database.
/// Important: Make sure `url` does not contain a trailing `/`
#[tracing::instrument]
pub fn init(url: &str) -> color_eyre::Result<(State, Receiver<MidEvent>)> {
    let (mut state, mut receiver) = State::new();
    url.clone_into(&mut state.url);

    let view_key = state.view_def(View {
        name: "Main View".to_string(),
        tasks: Some(state.tasks.keys().collect::<Vec<TaskKey>>()),
        ..View::default()
    });

    // request all tasks using a "None" filter into the default "Main View"
    state.spawn_request::<FilterRequest, FilterResponse>(
        state.client.get(format!("{url}/filter")),
        FilterRequest {
            filter: Filter::None,
            req_id: view_key.0.as_ffi(),
        },
    );

    Ok((state, receiver))
}

pub fn init_test() -> (State, Receiver<MidEvent>) {
    let (mut state, receiver) = State::new();
    let task1 = state.task_def(Task {
        name: "Eat Lunch".to_owned(),
        completed: true,
        ..Default::default()
    });
    let task2 = state.task_def(Task {
        name: "Finish ABN".to_owned(),
        ..Default::default()
    });
    let view_key = state.view_def(View {
        name: "Main View".to_string(),
        ..View::default()
    });
    state.view_mod(view_key, |v| v.tasks = Some(vec![task1, task2]));
    (state, receiver)
}

#[cfg(test)]
mod tests {
    pub use super::*;
    use chrono::{NaiveDate, NaiveDateTime};
    use common::backend::{
        DeleteTasksRequest, DeleteTasksResponse, FilterResponse, ReadTaskShortResponse,
    };
    use mockito::{Matcher, Server, ServerGuard};
    use serde_json::{to_value, to_vec};

    async fn mockito_setup() -> ServerGuard {
        let mut server = Server::new_async().await;

        server
            .mock("GET", "/filter")
            // .match_body(Matcher::Json(to_value(FilterRequest { filter: Filter::None, req_id: 0 }).unwrap()))
            .with_body_from_request(|req| {
                let req: FilterRequest =
                    serde_json::from_slice::<FilterRequest>(req.body().unwrap()).unwrap();
                to_vec(&FilterResponse {
                    tasks: vec![0, 1, 2],
                    req_id: req.req_id,
                })
                .unwrap()
            })
            .expect(1)
            .create_async()
            .await;

        server
            .mock("GET", "/tasks")
            //.match_body(Matcher::Json(to_value(&vec![0, 1, 2].into_iter().map(|task_id|ReadTaskShortRequest{task_id}).collect::<Vec<_>>()).unwrap()))
            .with_body(
                &to_vec::<ReadTasksShortResponse>(&vec![
                    Ok(ReadTaskShortResponse {
                        task_id: 0,
                        name: "Test Task 1".into(),
                        ..Default::default()
                    }),
                    Ok(ReadTaskShortResponse {
                        task_id: 1,
                        name: "Test Task 2".into(),
                        ..Default::default()
                    }),
                    Err("random error message".into()),
                    Ok(ReadTaskShortResponse {
                        task_id: 2,
                        name: "Test Task 3".into(),
                        ..Default::default()
                    }),
                ])
                .unwrap(),
            )
            .expect(1)
            .create_async()
            .await;

        server
            .mock("POST", "/task")
            .with_body_from_request(|req| {
                let req: CreateTaskRequest =
                    serde_json::from_slice::<CreateTaskRequest>(req.body().unwrap()).unwrap();
                to_vec(&CreateTaskResponse {
                    req_id: req.req_id,
                    task_id: 3,
                })
                .unwrap() // Note: This is mega sus b/c mock. Database ID is hardcoded!
            })
            .expect(1)
            .create_async()
            .await;

        server
            .mock("PUT", "/task")
            .with_body_from_request(|req| {
                let req = serde_json::from_slice::<UpdateTaskRequest>(req.body().unwrap()).unwrap();
                to_vec(&UpdateTaskResponse {
                    task_id: req.task_id,
                    req_id: req.req_id,
                })
                .unwrap()
            })
            .expect(1)
            .create_async()
            .await;

        server
            .mock("DELETE", "/task")
            // send back request
            .with_body_from_request(|req| {
                let req = serde_json::from_slice::<DeleteTaskRequest>(req.body().unwrap()).unwrap();
                println!("req is {:?}", req);
                let resp: DeleteTaskResponse = req.req_id;
                let new_resp = to_vec::<DeleteTaskResponse>(&resp).unwrap();

                println!("resp is {:?}", resp);
                new_resp
            })
            .expect(1)
            .create_async()
            .await;

        server
            .mock("GET", mockito::Matcher::Any)
            .with_body("TEST MAIN PATH")
            .expect(0)
            .create_async()
            .await;
        server
    }

    #[tokio::test]
    async fn test_do_request() {
        let mut server = Server::new_async().await;
        server
            .mock("GET", mockito::Matcher::Any)
            .with_body("TEST MAIN PATH")
            .expect(0)
            .create_async()
            .await;

        server
            .mock("GET", "/shouldincomplete")
            .with_body("invalid json")
            .expect(1)
            .create_async()
            .await;

        // create client
        let client = ClientBuilder::new(reqwest::Client::new())
            .with(TracingMiddleware::<SpanBackendWithUrl>::new())
            .build();

        // test can't connect err
        do_request::<_, FilterResponse>(
            client.get("localhost:1234/cantconnect"),
            FilterRequest {
                filter: Filter::None,
                req_id: 0,
            },
        )
        .await
        .unwrap_err();
        // test can't parse response err
        do_request::<_, FilterResponse>(
            client.get(format!("{}/shouldincomplete", server.url())),
            FilterRequest {
                filter: Filter::None,
                req_id: 0,
            },
        )
        .await
        .unwrap_err();
    }

    // tests the State init function, also used to init tests
    async fn test_init() -> (ServerGuard, State, Receiver<MidEvent>, ViewKey) {
        let server = mockito_setup().await;
        let url = server.url();
        println!("url: {url}");

        // init state
        let (mut state, mut receiver) = init(&url).unwrap();
        // await server response for FilterRequest
        state.handle_mid_event(receiver.next().await.unwrap());
        println!("ui event {:?}", receiver.next().await.unwrap()); // drop UI event
                                                                   // // await server response for ReadTasksShortResponse (request automatically sent when handle_mid_event is called on FilterResponse)
                                                                   //dbg!(receiver.next().await.unwrap());
        state.handle_mid_event(receiver.next().await.unwrap());
        println!("ui event {:?}", receiver.next().await.unwrap()); // drop UI event

        // make sure view was created with correct state
        let view_key = state.view_get_default().unwrap();
        let view = state.view_get(view_key).unwrap();
        let mut i = 0;
        view.tasks.as_ref().unwrap().iter().for_each(|t| {
            assert_eq!(state.task_get(*t).unwrap().db_id.unwrap(), i);
            i += 1;
        });

        (server, state, receiver, view_key)
    }

    #[tokio::test]
    // #[traced_test]
    async fn test_tasks() {
        let (server, mut state, mut receiver, view_key) = test_init().await;

        let view = state.view_get(view_key).unwrap();
        assert_eq!(view.name, "Main View");
        let mut tasks = view
            .tasks
            .as_ref()
            .unwrap()
            .iter()
            .cloned()
            .collect::<Vec<TaskKey>>();

        tasks.sort(); // make keys are in sorted order
                      // test task_mod
        state.task_mod(tasks[0], |t| "Eat Dinner".clone_into(&mut t.name));
        assert_eq!(state.task_get(tasks[0]).unwrap().name, "Eat Dinner");

        // test task_rm (& db key removal)
        // dbg!(receiver.next().await.unwrap()); // random error?
        state.task_rm(tasks[1]).unwrap();
        state.handle_mid_event(receiver.next().await.unwrap()); // the delete task event

        // test get function fail
        // dbg!(state.task_get(tasks[1]));
        state.task_get(tasks[1]).unwrap_err();

        // test mod function fail
        let mut test = 0;
        state.task_mod(tasks[1], |_| test = 1);
        assert_eq!(test, 0);

        // test update works
        state.task_mod(tasks[0], |t: &mut Task| {
            "Cook some lunch yo".clone_into(&mut t.name)
        });
        // dbg!(receiver.next().await.unwrap()); // skip state event
        state.handle_mid_event(receiver.next().await.unwrap());
        // dbg!(receiver.next().await.unwrap());
        // dbg!(receiver.next().await.unwrap());
        // dbg!(receiver.next().await.unwrap());
        //assert_eq!(1, 0);
        assert_eq!(state.task_get(tasks[0]).unwrap().name, "Cook some lunch yo");

        // test create task works
        let task1 = state.task_def(Task {
            name: "Eat Lunch".to_owned(),
            completed: true,
            ..Default::default()
        });
        // dbg!(receiver.next().await.unwrap()); // catch state event
        state.handle_mid_event(receiver.next().await.unwrap());
        assert_eq!(state.tasks[task1].name, "Eat Lunch");
    }

    #[tokio::test]
    async fn test_prop_def() {
        let (server, mut state, mut receiver, view_key) = test_init().await;

        let view = state.view_get(view_key).unwrap();
        assert_eq!(view.name, "Main View");
        let mut tasks = view
            .tasks
            .as_ref()
            .unwrap()
            .iter()
            .cloned()
            .collect::<Vec<TaskKey>>();

        tasks.sort(); // make keys are in sorted order

        // assign a date to tasks[0]
        let mut name_key = state.prop_def_name("Due Date");
        let mut prop_key = state
            .prop_def(
                tasks[0],
                name_key,
                TaskPropVariant::Date(NaiveDate::from_ymd_opt(2016, 7, 8).unwrap().into()),
            )
            .unwrap();
        let mut prop = &state.props[prop_key];

        // check that tasks 0 got assigned the date
        assert_eq!(prop, state.prop_get(tasks[0], name_key).unwrap());

        // test float prop
        name_key = state.prop_def_name("time to finish (seconds)");
        prop_key = state
            .prop_def(tasks[0], name_key, TaskPropVariant::Number(2.0567))
            .unwrap();
        prop = &state.props[prop_key];
        assert_eq!(prop, state.prop_get(tasks[0], name_key).unwrap());

        // test string prop
        name_key = state.prop_def_name("assignee");
        prop_key = state
            .prop_def(
                tasks[0],
                name_key,
                TaskPropVariant::String(String::from("yacobo")),
            )
            .unwrap();
        prop = &state.props[prop_key];
        assert_eq!(prop, state.prop_get(tasks[0], name_key).unwrap());

        // test bool prop
        name_key = state.prop_def_name("is ez?");
        prop_key = state
            .prop_def(tasks[0], name_key, TaskPropVariant::Boolean(false))
            .unwrap();
        prop = &state.props[prop_key];
        assert_eq!(prop, state.prop_get(tasks[0], name_key).unwrap());
    }

    #[tokio::test]
    async fn test_prop_mod() {
        let (server, mut state, mut receiver, view_key) = test_init().await;

        let view = state.view_get(view_key).unwrap();
        assert_eq!(view.name, "Main View");
        let mut tasks = view
            .tasks
            .as_ref()
            .unwrap()
            .iter()
            .cloned()
            .collect::<Vec<TaskKey>>();

        tasks.sort(); // make keys are in sorted order

        // String -> String mod
        let name_key = state.prop_def_name("random property");
        let prop_key = state
            .prop_def(
                tasks[0],
                name_key,
                TaskPropVariant::String(String::from("j")),
            )
            .unwrap();
        let prop_ref = &state.props[prop_key];

        state.prop_mod(tasks[0], name_key, |prop| match prop {
            TaskPropVariant::String(_) => {
                let mut new_prop = TaskPropVariant::String("jacob is cool".to_string());
                *prop = new_prop;
            }
            _ => {}
        });
        let new_random_str = state.prop_get(tasks[0], name_key).unwrap();
        let actual_random_str = match new_random_str {
            TaskPropVariant::String(s) => Some(s.as_str()),
            _ => None, // Handle other variants if needed
        };
        assert_eq!(actual_random_str.unwrap(), "jacob is cool");
    }

    #[tokio::test]
    async fn test_prop_rm() {
        let (server, mut state, mut receiver, view_key) = test_init().await;

        let view = state.view_get(view_key).unwrap();
        assert_eq!(view.name, "Main View");
        let mut tasks = view
            .tasks
            .as_ref()
            .unwrap()
            .iter()
            .cloned()
            .collect::<Vec<TaskKey>>();

        tasks.sort(); // make keys are in sorted order

        // assign a date to tasks[0]
        let mut name_key = state.prop_def_name("Due Date");
        let mut prop_key = state
            .prop_def(
                tasks[0],
                name_key,
                TaskPropVariant::Date(NaiveDate::from_ymd_opt(2016, 7, 8).unwrap().into()),
            )
            .unwrap();
        let mut prop = &state.props[prop_key];

        // remove the date
        state.prop_rm(tasks[0], name_key);
        assert!(state.prop_get(tasks[0], name_key).is_err()); // should throw err
    }

    #[tokio::test]
    async fn test_remove_prop_name_deletes_props_prop_map_and_props() {
        let (server, mut state, mut receiver, view_key) = test_init().await;

        let view = state.view_get(view_key).unwrap();
        assert_eq!(view.name, "Main View");
        let mut tasks = view
            .tasks
            .as_ref()
            .unwrap()
            .iter()
            .cloned()
            .collect::<Vec<TaskKey>>();

        tasks.sort(); // make keys are in sorted order

        // assign a date to tasks[0]
        let mut name_key = state.prop_def_name("Due Date");
        state
            .prop_def(
                tasks[0],
                name_key,
                TaskPropVariant::Date(NaiveDate::from_ymd_opt(2016, 7, 8).unwrap().into()),
            )
            .unwrap();
        state
            .prop_def(
                tasks[1],
                name_key,
                TaskPropVariant::Date(NaiveDate::from_ymd_opt(2016, 7, 8).unwrap().into()),
            )
            .unwrap();
        state
            .prop_def(
                tasks[2],
                name_key,
                TaskPropVariant::Date(NaiveDate::from_ymd_opt(2016, 7, 8).unwrap().into()),
            )
            .unwrap();

        state.prop_rm_name(name_key);
        assert!(state.prop_get(tasks[0], name_key).is_err()); // these should all be errors
        assert!(state.prop_get(tasks[1], name_key).is_err());
        assert!(state.prop_get(tasks[1], name_key).is_err());
    }

    // #[tokio::test]
    // async fn test_remove_prop_name_deletes_props_prop_map_and_props() {
    //     todo!();
    // }

    #[tokio::test]
    async fn test_prop_def_twice() {
        let (server, mut state, mut receiver, view_key) = test_init().await;

        let view = state.view_get(view_key).unwrap();
        assert_eq!(view.name, "Main View");
        let mut tasks = view
            .tasks
            .as_ref()
            .unwrap()
            .iter()
            .cloned()
            .collect::<Vec<TaskKey>>();

        tasks.sort(); // make keys are in sorted order

        // assign a date to tasks[0]
        let name_key = state.prop_def_name("random property");
        let old_prop_key = state
            .prop_def(
                tasks[0],
                name_key,
                TaskPropVariant::Date(NaiveDate::from_ymd_opt(2016, 7, 8).unwrap().into()),
            )
            .unwrap();
        let prop_key = state
            .prop_def(
                tasks[0],
                name_key,
                TaskPropVariant::String(String::from("j")),
            )
            .unwrap();
        let old_prop_ref = &state.props[old_prop_key];
        let new_prop_ref = &state.props[prop_key];

        // both should have type "string"
        //assert_eq!(old_prop_ref.type_string(), new_prop_ref.type_string());
    }
    #[tokio::test]
    async fn test_view_task_keys() {
        let (server, mut state, mut receiver, view_key) = test_init().await;
        let task_key_iter = state.view_task_keys(view_key).unwrap();
        assert_eq!(task_key_iter.clone().count(), 3);
        //let task_key1 = task_key_iter.clone().next();
        //assert_eq!(task_key1, 1);
        //let task_key2 =
        //assert_eq!(1, 0);
    }

    #[tokio::test]
    async fn test_view_tasks() {
        let (server, mut state, mut receiver, view_key) = test_init().await;
        let task_key_iter = state.view_tasks(view_key).unwrap();
        assert_eq!(task_key_iter.clone().count(), 3);
        //let task_key1 = task_key_iter.clone().next();
        //assert_eq!(task_key1, 1);
        //let task_key2 =
        //assert_eq!(1, 0);
    }

    /* #[tokio::test]
    async fn test_frontend_api() {
        let server = mockito_setup().await;
        let url = server.url();
        println!("url: {url}");



        let name_key = state.prop_def_name("Due Date");
        // test prop def removal
        let invalid_name_key = state.prop_def_name("Invalid");
        assert_eq!(state.prop_rm_name(invalid_name_key).unwrap(), "Invalid");
        assert!(state.prop_rm_name(invalid_name_key).is_err());

        // test prop_def
        state
            .prop_def(tasks[0], name_key, TaskPropVariant::Boolean(false))
            .unwrap();
        assert!(state
            .prop_def(tasks[0], invalid_name_key, TaskPropVariant::Boolean(false))
            .is_err());
        assert!(state
            .prop_def(tasks[1], name_key, TaskPropVariant::Boolean(false))
            .is_err());

        // test prop_mod
        assert!(state
            .prop_mod(tasks[1], name_key, |t| *t = TaskPropVariant::Boolean(true))
            .is_err());
        assert!(state
            .prop_mod(tasks[0], invalid_name_key, |t| *t =
                TaskPropVariant::Boolean(true))
            .is_err());
        assert!(state
            .prop_mod(tasks[0], name_key, |t| *t = TaskPropVariant::Boolean(true))
            .is_ok());
        // test prop_get
        assert_eq!(
            state.prop_get(tasks[0], name_key).unwrap(),
            &TaskPropVariant::Boolean(true)
        );
        assert!(state.prop_get(tasks[1], name_key).is_err());
        assert!(state.prop_get(tasks[0], invalid_name_key).is_err());

        // test prop_rm
        assert!(state.prop_rm(tasks[0], invalid_name_key).is_err());
        assert!(state.prop_rm(tasks[1], name_key).is_err());
        assert_eq!(
            state.prop_rm(tasks[0], name_key).unwrap(),
            TaskPropVariant::Boolean(true)
        );
        assert!(state.prop_rm(tasks[0], name_key).is_err());

        // script testing
        let script_id = state.script_create();
        state.script_mod(script_id, |s| {
            "function do_lua()".clone_into(&mut s.content)
        });
        assert_eq!(
            state.script_get(script_id).unwrap().content,
            "function do_lua()"
        );
        state.script_rm(script_id);
        assert!(state.script_get(script_id).is_none());

        // test remove view
        state.view_rm(view_key);
        assert!(state.view_get(view_key).is_err());

        // prop errors
        dbg!(PropDataError::Prop(tasks[0], name_key));
        println!("{}", PropDataError::Prop(tasks[1], invalid_name_key));
    } */
    #[test]
    fn test_view_new() {
        let expected_view = super::View {
            name: "test".to_owned(),
            ..Default::default()
        };
        let new_view = super::View::new("test".to_string());
        assert_eq!(expected_view, new_view)
    }
    #[test]
    fn test_new_task() {
        let expected_task = super::Task {
            name: "test".to_owned(),
            completed: false.to_owned(),
            ..Default::default()
        };
        let new_task = super::Task::new("test".to_string(), false);
        assert_eq!(expected_task, new_task)
    }
    #[test]
    #[should_panic]
    fn state_handle_mid_event_fails_on_state_event() {
        let (mut state, _) = super::State::new();
        assert!(state
            .handle_mid_event(super::MidEvent::StateEvent(super::StateEvent::TasksUpdate))
            .is_err());
    }
    #[tokio::test]
    async fn test_task_rm_removes_task_if_not_in_db() {
        let (mut state, _) = super::State::new();
        let task_key = state.task_def(super::Task::default());
        state.tasks.get_mut(task_key).unwrap().db_id = None;
        state.task_rm(task_key).unwrap();
        assert!(state.tasks.get(task_key).is_none());
    }
}
#[cfg(test)]
mod derive_tests {
    use tracing_subscriber::fmt::format;

    #[test]
    fn test_no_task_error_debug_clone() {
        let error = super::NoTaskError(super::TaskKey::default());
        let clone = error.clone();
        let debug_string = format!("{:?}", error);
    }
    #[test]
    fn test_unsyncronized_task_error_debug_clone() {
        let error = super::UnsyncronizedTaskError(super::TaskKey::default());
        let clone = error.clone();
        let debug_string = format!("{:?}", error);
    }
    #[test]
    fn test_no_view_error_debug_clone() {
        let error = super::NoViewError(super::ViewKey::default());
        let clone = error.clone();
        let debug_string = format!("{:?}", error);
    }
}
