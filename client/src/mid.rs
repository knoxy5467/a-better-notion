//! Middleware Logic
#![allow(unused)]

use common::*;

use serde::{Deserialize, Serialize};
use slotmap::{new_key_type, SlotMap};
use std::collections::HashMap;

new_key_type! { struct PropKey; }
new_key_type! { struct TaskKey; }

/// All data associated with tasks, except for properties
pub struct Task {
    /// contains a TaskShort (contains description and other things)
    short: TaskShort,
    /// dependency list on other tasks
    deps: Vec<TaskKey>,
    /// if it is stored in the database, it will have a unique task_id.
    db_id: Option<TaskID>,
}

new_key_type! { pub struct PropNameKey; }

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
    views: HashMap<ViewID, View>,
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
trait FrontendAPI {
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
    fn view_def(&mut self, view: View) -> ViewID;
    fn view_get(&self, view_id: ViewID) -> Option<&View>;
    fn view_mod(&mut self, view_id: ViewID, edit_fn: impl FnOnce(&mut View)) -> Option<()>;
    fn view_rm(&mut self, view_id: ViewID);

    /// create/get/modify script data.
    fn script_create(&mut self) -> ScriptID;
    fn script_get(&self, view_id: ScriptID) -> &View;
    fn script_mod(&mut self, view_id: ScriptID, edit_fn: impl FnOnce(&mut Script));
    fn script_rm(&mut self, view_id: ScriptID);

    /// register ui events with middleware, (i.e. so scripts can run when they are triggered)
    fn register_event(&mut self, name: &str);
    /// notify middleware of registered event
    fn event_notify(&mut self, name: &str) -> bool;
}

impl FrontendAPI for State {
    fn task_def(&mut self, task: Task) -> TaskKey {
        self.tasks.insert(task)
        // TODO: register definition to queue so that we can sync to server
        
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

    fn view_def(&mut self, view: View) -> ViewID {
        todo!()
    }

    fn view_get(&self, view_id: ViewID) -> Option<&View> {
        todo!()
    }

    fn view_mod(&mut self, view_id: ViewID, edit_fn: impl FnOnce(&mut View)) -> Option<()> {
        todo!()
    }

    fn view_rm(&mut self, view_id: ViewID) {
        todo!()
    }

    fn script_create(&mut self) -> ScriptID {
        todo!()
    }

    fn script_get(&self, view_id: ScriptID) -> &View {
        todo!()
    }

    fn script_mod(&mut self, view_id: ScriptID, edit_fn: impl FnOnce(&mut Script)) {
        todo!()
    }

    fn script_rm(&mut self, view_id: ScriptID) {
        todo!()
    }

    fn register_event(&mut self, name: &str) {
        todo!()
    }

    fn event_notify(&mut self, name: &str) -> bool {
        todo!()
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

/* fn test() {
    init("test").
} */
