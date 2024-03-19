//! Middleware Logic
#![allow(unused)]

use common::*;

use serde::{Deserialize, Serialize};
use slotmap::{new_key_type, SlotMap};
use std::collections::HashMap;

new_key_type! { struct PropKey; }
new_key_type! { struct TaskKey; }

pub struct Task {
    short: TaskShort,
    deps: Vec<TaskKey>,
}

/// State
#[derive(Default)]
pub struct State {
    task_map: HashMap<TaskID, TaskKey>,
    tasks: SlotMap<TaskKey, Task>,
    /// map property names to slotmap ids
    prop_map: HashMap<(TaskID, String), PropKey>,
    /// efficient, dense storage of all locally-stored task properties
    props: SlotMap<PropKey, TaskPropVariant>,
    /// scripts
    scripts: HashMap<ScriptID, Script>,
    /// view data
    views: HashMap<ViewID, View>,
    /// connected url
    url: String,
}

pub enum PropDataError {
    InvalidTask,
    UndefinedProp(String),
}

enum ScriptEvent {
    RegisteredEvent(&'static str),
}

/// Event sent to UI via channel to notify UI that some data has changed and the render should be updated.
enum StateUpdate {
    TaskUpdate(TaskKey),
    PropUpdate(PropKey),
    ViewUpdate(ViewID),
    ScriptUpdate(ScriptID),
    MultiStateUpdate,
    ServerStatusUpdate(bool),
}

/// Frontend API Trait
trait FrontendAPI {
    /// create/view/modify tasks
    fn task_def(&mut self, task: Task) -> TaskKey;
    fn task_get(&self) -> Option<&Task>;
    fn task_mod(&mut self, key: TaskKey, edit_fn: impl FnOnce(&mut TaskShort));
    fn task_rm(&mut self, key: TaskKey);

    /// create/view/modify task properties
    fn prop_def(&mut self, task_key: TaskKey, name: &str, prop: TaskPropVariant) -> PropKey;
    fn prop_mod(&mut self, task_key: TaskKey, name: &str, edit_fn: impl FnOnce(&mut TaskPropVariant)) -> Option<()>;
    fn prop_get(&self, task_key: TaskKey, string: &str) -> Result<&TaskPropVariant, PropDataError>;
    fn prop_rm(&mut self, task_key: TaskKey, name: &str);

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
        todo!()
    }

    fn task_get(&self) -> Option<&Task> {
        todo!()
    }

    fn task_mod(&mut self, key: TaskKey, edit_fn: impl FnOnce(&mut TaskShort)) {
        todo!()
    }

    fn task_rm(&mut self, key: TaskKey) {
        todo!()
    }

    fn prop_def(&mut self, task_key: TaskKey, name: &str, prop: TaskPropVariant) -> PropKey {
        todo!()
    }

    fn prop_mod(&mut self, task_key: TaskKey, name: &str, edit_fn: impl FnOnce(&mut TaskPropVariant)) -> Option<()> {
        todo!()
    }

    fn prop_get(&self, task_key: TaskKey, string: &str) -> Result<&TaskPropVariant, PropDataError> {
        todo!()
    }

    fn prop_rm(&mut self, task_key: TaskKey, name: &str) {
        todo!()
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
