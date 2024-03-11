use serde::{Deserialize, Serialize};
use slotmap::SlotMap;

<<<<<<< HEAD
=======
use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use slotmap::{new_key_type, SlotMap};

/// local unique property id for storing props in dense slotmap
new_key_type! { struct PropID; }

/// State
struct State {
    tasks: HashMap<TaskID, TaskShort>,
	/// map property names to slotmap ids
	prop_names: HashMap<String, PropID>,
	/// efficient, dense storage of all locally-stored task properties
	props: SlotMap<PropID, TaskPropVariant>,
	/// scripts
	scripts: HashMap<ScriptID, Script>,
	/// view data
	views: HashMap<ViewID, View>,
}

>>>>>>> e152537c83b1d7dd0bb05d17f73e8ab01bf7121d
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
enum TaskPropVariant {
    Date(()),
    String(String),
    Number(f64),
    Boolean(bool),
}
#[derive(Serialize, Deserialize)]
struct TaskProp {
    name: String,
    value: TaskPropVariant,
}

<<<<<<< HEAD
type ScriptID = u64;
slotmap::new_key_type! { struct TaskID; }
slotmap::new_key_type! { struct PropertyName;}
pub struct Task {
=======
struct TaskShort {
	/// DB Primary Key
>>>>>>> e152537c83b1d7dd0bb05d17f73e8ab01bf7121d
    task_id: TaskID,
	/// Short name of the task (description is a property)
    name: String,
	/// Whether the task is completed or not
    completed: bool,
<<<<<<< HEAD
    properties: SlotMap<PropertyName, TaskPropVariant>,
    dependencies: Vec<Task>,
    scripts: Vec<ScriptID>,
}

/// State
struct State {
    tasks: SlotMap<TaskID, Task>,
    // other state stored by UI
    // ui: UIState,
}

// BACKEND API
/// reqwest::post("/task").body(CreateTaskRequest {})
#[derive(Serialize, Deserialize)]
struct CreateTaskRequest {
    name: String,
    completed: bool,
    properties: Vec<TaskProp>,
    /// [name, date, value]
    dependencies: Vec<TaskID>,
}
/// reqwest::post("/tasks").body(CreateTaskRequest {})
type CreateTasksRequest = Vec<CreateTaskRequest>;
type CreateTaskResponse = Vec<TaskID>;

/// reawest::get("/task")
struct ReadTaskShortRequest {
    task_id: TaskID,
}
struct ReadTaskShortResponse {
    task_id: TaskID,
    name: String,
    completed: bool,
    props: Vec<String>,
    deps: Vec<TaskID>,
=======
	/// Dependencies of this task
    dependencies: Vec<TaskID>,
	/// Associated scripts
>>>>>>> e152537c83b1d7dd0bb05d17f73e8ab01bf7121d
    scripts: Vec<ScriptID>,
}
