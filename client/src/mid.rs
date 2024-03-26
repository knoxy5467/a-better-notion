//! Middleware Logic
#![allow(unused)]

use common::*;

use serde::{Deserialize, Serialize};
use slotmap::{new_key_type, SlotMap};
use std::collections::HashMap;

new_key_type! { pub struct PropKey; }
new_key_type! { pub struct TaskKey; }

/// All data associated with tasks, except for properties
#[derive(Debug, Default)]
pub struct Task {
    /// Short name of the task (description is a property)
    pub name: String,
    /// Whether the task is completed or not
    pub completed: bool,
    /// Dependencies of this task
    pub dependencies: Vec<TaskID>,
    /// Associated scripts
    pub scripts: Vec<ScriptID>,
    /// if it is stored in the database, it will have a unique task_id.
    pub db_id: Option<TaskID>,
}

/// Middleware stored View
#[derive(Debug, Default)]
pub struct View {
    /// Filter for view
    pub filter: Filter,
    /// Properties shown in view
    pub props: Vec<String>,
    /// Computed task list for view
    pub db_id: Option<ViewID>,
    pub tasks: Option<Vec<TaskKey>>,
}

new_key_type! { pub struct PropNameKey; }
new_key_type! { pub struct ViewKey; }

/// Middleware State structure.
#[derive(Default)]
pub struct State {
    /// maps between database ID and middleware ID for task
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
    /// Connection status
    status: bool,
}

/// Error returned if property does not exist
pub enum PropDataError {
    /// Property does not exist because associated task does not exist.
    InvalidTask,
    /// Property name does not exist.
    UndefinedProp(String),
}

/// Anything in this enum is sent to the middleware script executor when a UI event is triggered.
enum ScriptEvent { 
    /// Name of the event
    RegisteredEvent(String),
}

/// Event sent to UI via channel to notify UI that some data has changed and the render should be updated.
enum StateEvent {
    /// A task's core data was updated (not triggered for property updates)
    TaskUpdate(TaskKey),
    /// A property was updated
    PropUpdate(PropKey),
    /// A view configuration
    ViewUpdate(ViewID),
    /// A script was updated
    ScriptUpdate(ScriptID),
    /// Too much state has changed, UI should re-render everything.
    MultiState,
    /// The connection has either connected or disconnected.
    ServerStatus(bool),
}

/// Frontend API Trait
pub trait FrontendAPI {

    /// create/view/modify tasks
    fn task_def(&mut self, task: Task) -> TaskKey;
    fn task_get(&self, key: TaskKey) -> Option<&Task>;
    fn task_mod(&mut self, key: TaskKey, edit_fn: impl FnOnce(&mut Task));
    fn task_rm(&mut self, key: TaskKey);

    /// create/view/modify task properties
    fn prop_def(&mut self, task_key: TaskKey, name: PropNameKey, prop: TaskPropVariant) -> PropKey;
    fn prop_get(&self, task_key: TaskKey, name: PropNameKey) -> Result<&TaskPropVariant, PropDataError>;
    fn prop_mod(
        &mut self,
        task_key: TaskKey,
        name: PropNameKey,
        edit_fn: impl FnOnce(&mut TaskPropVariant),
    ) -> Result<(), PropDataError>;
    fn prop_rm(&mut self, task_key: TaskKey, name: PropNameKey) -> Result<TaskPropVariant, PropDataError>;

    /// create/get/modify views
    fn view_def(&mut self, view: View) -> ViewKey;
    fn view_get(&self, view_key: ViewKey) -> Option<&View>;
    fn view_tasks(&self, view_key: ViewKey) -> Option<&[TaskKey]>;
    fn view_mod(&mut self, view_key: ViewKey, edit_fn: impl FnOnce(&mut View)) -> Option<()>;
    fn view_rm(&mut self, view_key: ViewKey);

    /// create/get/modify script data.
    fn script_create(&mut self) -> ScriptID;
    fn script_get(&self, script_id: ScriptID) -> &View;
    fn script_mod(&mut self, script_id: ScriptID, edit_fn: impl FnOnce(&mut Script));
    fn script_rm(&mut self, script_id: ScriptID);

    /// register ui events with middleware, (i.e. so scripts can run when they are triggered)
    fn register_event(&mut self, name: &str);
    /// notify middleware of registered event
    fn event_notify(&mut self, name: &str) -> bool;
}

impl FrontendAPI for State {
    fn task_def(&mut self, task: Task) -> TaskKey {
        let key = self.tasks.insert(task);
        // TODO: register definition to queue so that we can sync to server
        key
    }

    fn task_get(&self, key: TaskKey) -> Option<&Task> {
        self.tasks.get(key)
    }

    fn task_mod(&mut self, key: TaskKey, edit_fn: impl FnOnce(&mut Task)) {
        if let Some(task) = self.tasks.get_mut(key) { edit_fn(task) }
    }

    fn task_rm(&mut self, key: TaskKey) {
        if let Some(db_id) = self.tasks.remove(key).and_then(|t|t.db_id) {
            self.task_map.remove(&db_id);
        }
    }

    fn prop_def(&mut self, task_key: TaskKey, name: PropNameKey, prop: TaskPropVariant) -> PropKey {
        let prop_key = self.props.insert(prop);
        self.prop_map.insert((task_key, name.to_owned()), prop_key);
        prop_key
    }

    fn prop_get(&self, task_key: TaskKey, name: PropNameKey) -> Result<&TaskPropVariant, PropDataError> {
        let key = self.prop_map.get(&(task_key, name)).ok_or_else(||{
            if self.tasks.contains_key(task_key) { PropDataError::UndefinedProp(self.prop_names[name].clone()) }
            else { PropDataError::InvalidTask }
        })?;
        Ok(&self.props[*key])
    }

    fn prop_mod(
        &mut self,
        task_key: TaskKey,
        name: PropNameKey,
        edit_fn: impl FnOnce(&mut TaskPropVariant),
    ) -> Result<(), PropDataError> {
        let key = self.prop_map.get(&(task_key, name)).ok_or_else(||{
            if self.tasks.contains_key(task_key) { PropDataError::UndefinedProp(self.prop_names[name].clone()) }
            else { PropDataError::InvalidTask }
        })?;
        edit_fn(&mut self.props[*key]);
        Ok(())
    }

    fn prop_rm(&mut self, task_key: TaskKey, name: PropNameKey) -> Result<TaskPropVariant, PropDataError> {
        let key = self.prop_map.get(&(task_key, name)).ok_or_else(||{
            if self.tasks.contains_key(task_key) { PropDataError::UndefinedProp(self.prop_names[name].clone()) }
            else { PropDataError::InvalidTask }
        })?;
        self.props.remove(*key).ok_or_else(||PropDataError::UndefinedProp(self.prop_names[name].clone()))
    }

    fn view_def(&mut self, view: View) -> ViewKey {
        let key = self.views.insert(view);
        // TODO: register to save updated view
        key
    }

    fn view_get(&self, view_key: ViewKey) -> Option<&View> {
        self.views.get(view_key)
    }

    fn view_tasks(&self, view_key: ViewKey) -> Option<&[TaskKey]> {
        self.views.get(view_key).and_then(|v|v.tasks.as_ref()).map(|v|v.as_slice())
    }

    fn view_mod(&mut self, view_key: ViewKey, edit_fn: impl FnOnce(&mut View)) -> Option<()> {
        let mut view = self.views.get_mut(view_key)?;
        edit_fn(&mut view);
        None
    }

    fn view_rm(&mut self, view_key: ViewKey) {
        todo!()
    }

    fn script_create(&mut self) -> ScriptID {
        todo!()
    }

    fn script_get(&self, view_key: ScriptID) -> &View {
        todo!()
    }

    fn script_mod(&mut self, view_key: ScriptID, edit_fn: impl FnOnce(&mut Script)) {
        todo!()
    }

    fn script_rm(&mut self, view_key: ScriptID) {
        todo!()
    }

    fn register_event(&mut self, name: &str) {
        todo!()
    }

    fn event_notify(&mut self, name: &str) -> bool {
        todo!()
    }
}

impl State {
    pub fn tasks<'a>(&'a self) -> impl Iterator<Item = &'a Task> {
        self.tasks.iter().map(|(a, b)| b)
    }
}

/// Init middleware state
/// This function is called by UI to create the Middleware state and establish a connection to the Database.
pub async fn init(url: &str) -> Result<State, reqwest::Error> {
    let state = State {
        url: url.to_owned(),
        ..Default::default()
    };

    let client = reqwest::Client::new();
    client.execute(client.post(&state.url).build()?).await?;

    Ok(state)
}

pub fn init_example() -> (State, ViewKey) {
    let mut state = State::default();
    let task1 = state.task_def(Task { name: "Eat Lunch".to_owned(), completed: true, ..Default::default() });
    let task2 = state.task_def(Task { name: "Finish ABN".to_owned(), ..Default::default() });
    let view_key = state.view_def(View { ..View::default() });
    state.view_mod(view_key, |v|v.tasks = Some(vec![task1, task2]));
    (state, view_key)
}

/* fn test() {
    init("test").
} */
