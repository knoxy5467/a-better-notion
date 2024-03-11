use serde::{Deserialize, Serialize};
use slotmap::SlotMap;

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

struct TaskShort {
	/// DB Primary Key
    task_id: TaskID,
	/// Short name of the task (description is a property)
    name: String,
	/// Whether the task is completed or not
    completed: bool,
	/// Dependencies of this task
    dependencies: Vec<TaskID>,
	/// Associated scripts
    scripts: Vec<ScriptID>,
}
