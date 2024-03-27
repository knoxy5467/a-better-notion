//! Middleware Logic
#![allow(unused)]

use common::*;

use serde::{Deserialize, Serialize};
use slotmap::{new_key_type, SlotMap};
use thiserror::Error;
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

/* /// Frontend API Trait
pub trait FrontendAPI {

    /// create/view/modify tasks
    fn task_def(&mut self, task: Task) -> TaskKey;
    fn task_get(&self, key: TaskKey) -> Option<&Task>;
    fn task_mod(&mut self, key: TaskKey, edit_fn: impl FnOnce(&mut Task));
    fn task_rm(&mut self, key: TaskKey);

    /// create/view/modify task properties
    fn prop_def(&mut self, task_key: TaskKey, name_key: PropNameKey, prop: TaskPropVariant) -> Result<PropKey, PropDataError>;
    fn prop_def_name(&mut self, name: impl Into<String>) -> PropNameKey;
    fn prop_rm_name(&mut self, key: PropNameKey);
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
    fn script_get(&self, script_id: ScriptID) -> Option<&Script>;
    fn script_mod(&mut self, script_id: ScriptID, edit_fn: impl FnOnce(&mut Script));
    fn script_rm(&mut self, script_id: ScriptID);

    /// register ui events with middleware, (i.e. so scripts can run when they are triggered)
    fn register_event(&mut self, name: &str);
    /// notify middleware of registered event
    fn event_notify(&mut self, name: &str) -> bool;
} */

impl State {
    pub fn task_def(&mut self, task: Task) -> TaskKey {
        // TODO: register definition to queue so that we can sync to server
        self.tasks.insert(task)
    }

    pub fn task_get(&self, key: TaskKey) -> Option<&Task> {
        self.tasks.get(key)
    }

    pub fn task_mod(&mut self, key: TaskKey, edit_fn: impl FnOnce(&mut Task)) {
        if let Some(task) = self.tasks.get_mut(key) { edit_fn(task) }
    }

    pub fn task_rm(&mut self, key: TaskKey) {
        if let Some(db_id) = self.tasks.remove(key).and_then(|t|t.db_id) {
            self.task_map.remove(&db_id);
        }
    }

    pub fn prop_def(&mut self, task_key: TaskKey, name_key: PropNameKey, prop: TaskPropVariant) -> Result<PropKey, PropDataError> {
        let Some(_) = self.tasks.get(task_key)
        else { return Err(PropDataError::Task(task_key)); };
        let Some(_) = self.prop_names.get(name_key)
        else { return Err(PropDataError::PropertyName(name_key)); };

        let prop_key = self.props.insert(prop);
        self.prop_map.insert((task_key, name_key), prop_key);
        Ok(prop_key)
    }
    pub fn prop_def_name(&mut self, name: impl Into<String>) -> PropNameKey {
        let name: String = name.into();
        let key = self.prop_names.insert(name.clone());
        self.prop_name_map.insert(name, key);
        key
    }
    pub fn prop_rm_name(&mut self, name_key: PropNameKey) -> Result<String, PropDataError> {
        let name = self.prop_names.remove(name_key).ok_or(PropDataError::PropertyName(name_key))?;
        self.prop_name_map.remove(&name);
        Ok(name)
    }

    pub fn prop_get(&self, task_key: TaskKey, name_key: PropNameKey) -> Result<&TaskPropVariant, PropDataError> {
        let Some(_) = self.tasks.get(task_key)
        else { return Err(PropDataError::Task(task_key)); };
        let Some(_) = self.prop_names.get(name_key)
        else { return Err(PropDataError::PropertyName(name_key)); };

        let key = self.prop_map.get(&(task_key, name_key)).ok_or(PropDataError::Prop(task_key, name_key))?;
        Ok(&self.props[*key])
    }

    pub fn prop_mod(
        &mut self,
        task_key: TaskKey,
        name_key: PropNameKey,
        edit_fn: impl FnOnce(&mut TaskPropVariant),
    ) -> Result<(), PropDataError> {
        let Some(_) = self.tasks.get(task_key)
        else { return Err(PropDataError::Task(task_key)); };
        let Some(_) = self.prop_names.get(name_key)
        else { return Err(PropDataError::PropertyName(name_key)); };

        let key = self.prop_map.get(&(task_key, name_key)).ok_or(PropDataError::Prop(task_key, name_key))?;
        edit_fn(self.props.get_mut(*key).ok_or(PropDataError::Prop(task_key, name_key))?);
        Ok(())
    }

    pub fn prop_rm(&mut self, task_key: TaskKey, name_key: PropNameKey) -> Result<TaskPropVariant, PropDataError> {
        let Some(_) = self.tasks.get(task_key)
        else { return Err(PropDataError::Task(task_key)); };
        let Some(_) = self.prop_names.get(name_key)
        else { return Err(PropDataError::PropertyName(name_key)); };

        let key = self.prop_map.remove(&(task_key, name_key)).ok_or(PropDataError::Prop(task_key, name_key))?;
        self.props.remove(key).ok_or(PropDataError::Prop(task_key, name_key))
    }

    pub fn view_def(&mut self, view: View) -> ViewKey {
        // TODO: register to save updated view
        self.views.insert(view)
    }

    pub fn view_get(&self, view_key: ViewKey) -> Option<&View> {
        self.views.get(view_key)
    }

    pub fn view_tasks(&self, view_key: ViewKey) -> Option<&[TaskKey]> {
        self.views.get(view_key).and_then(|v|v.tasks.as_ref()).map(|v|v.as_slice())
    }

    pub fn view_mod(&mut self, view_key: ViewKey, edit_fn: impl FnOnce(&mut View)) -> Option<()> {
        edit_fn(self.views.get_mut(view_key)?);
        None
    }

    pub fn view_rm(&mut self, view_key: ViewKey) {
        self.views.remove(view_key);
    }

    pub fn script_create(&mut self) -> ScriptID {
        self.scripts.insert(0, Script::default());
        0
    }

    pub fn script_get(&self, script_id: ScriptID) -> Option<&Script> {
        self.scripts.get(&script_id)
    }

    pub fn script_mod(&mut self, script_id: ScriptID, edit_fn: impl FnOnce(&mut Script)) {
        self.scripts.entry(script_id).and_modify(edit_fn);
    }

    pub fn script_rm(&mut self, script_id: ScriptID) {
        self.scripts.remove(&script_id);
    }

    /* pub fn register_event(&mut self, name: &str) {
        todo!()
    }

    pub fn event_notify(&mut self, name: &str) -> bool {
        todo!()
    } */
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
    let view_key = state.view_def(View { name: "Main View".to_string(), ..View::default() });
    state.view_mod(view_key, |v|v.tasks = Some(vec![task1, task2]));
    (state, view_key)
}

#[cfg(test)]
mod tests {
    pub use super::*;

    #[tokio::test]
    async fn test_init() {
        assert!(init("http:localhost:69420").await.is_err())
    }

    #[test]
    fn test_frontend_api() {
        // test view_def, view_mod & task_def
        let (mut state, view_key) = init_example();
        // test view_get
        let view = state.view_get(view_key).unwrap();
        assert_eq!(view.name, "Main View");

        let tasks = view.tasks.as_ref().unwrap().clone();
        // test task_mod
        state.task_mod(tasks[0], |t|t.name = "Eat Dinner".to_owned());
        assert_eq!(state.task_get(tasks[0]).unwrap().name, "Eat Dinner");

        // test task_rm (& db key removal)
        state.task_map.insert(0, tasks[1]);
        state.tasks[tasks[1]].db_id = Some(0);
        state.task_rm(tasks[1]);
        assert!(state.task_get(tasks[1]).is_none());
        assert!(state.task_map.is_empty());
        
        let name_key = state.prop_def_name("Due Date");
        // test prop def removal
        let invalid_name_key = state.prop_def_name("Invalid");
        assert_eq!(state.prop_rm_name(invalid_name_key).unwrap(), "Invalid");
        assert!(state.prop_rm_name(invalid_name_key).is_err());

        // test prop_def
        state.prop_def(tasks[0], name_key, TaskPropVariant::Boolean(false));
        assert!(state.prop_def(tasks[0], invalid_name_key, TaskPropVariant::Boolean(false)).is_err());
        assert!(state.prop_def(tasks[1], name_key, TaskPropVariant::Boolean(false)).is_err());
        
        // test prop_mod
        assert!(state.prop_mod(tasks[1], name_key, |t|*t = TaskPropVariant::Boolean(true)).is_err());
        assert!(state.prop_mod(tasks[0], invalid_name_key, |t|*t = TaskPropVariant::Boolean(true)).is_err());
        assert!(state.prop_mod(tasks[0], name_key, |t|*t = TaskPropVariant::Boolean(true)).is_ok());
        // test prop_get
        assert_eq!(state.prop_get(tasks[0], name_key).unwrap(), &TaskPropVariant::Boolean(true));
        assert!(state.prop_get(tasks[1], name_key).is_err());
        assert!(state.prop_get(tasks[0], invalid_name_key).is_err());

        // test prop_rm
        assert!(state.prop_rm(tasks[0], invalid_name_key).is_err());
        assert!(state.prop_rm(tasks[1], name_key).is_err());
        assert_eq!(state.prop_rm(tasks[0], name_key).unwrap(), TaskPropVariant::Boolean(true));
        assert!(state.prop_rm(tasks[0], name_key).is_err());

        /// script testing
        let script_id = state.script_create();
        state.script_mod(script_id, |s|s.content = "function do_lua()".to_owned());
        assert_eq!(state.script_get(script_id).unwrap().content, "function do_lua()");
        state.script_rm(script_id);
        assert!(state.script_get(script_id).is_none());

        /// test remove view
        state.view_rm(view_key);
        assert!(state.view_get(view_key).is_none());

        // prop errors
        dbg!(PropDataError::Prop(tasks[0], name_key));
        println!("{}", PropDataError::Prop(tasks[1], invalid_name_key));
    }
}