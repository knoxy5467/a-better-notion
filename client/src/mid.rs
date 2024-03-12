use common::{
    Comparator, Filter, Operator, PropID, Script, ScriptID, TaskID, TaskProp, TaskPropVariant,
    View, ViewID,
};
use serde::{Deserialize, Serialize};
use slotmap::{new_key_type, SlotMap};
use std::collections::HashMap;

/// local unique property id for storing props in dense slotmap
new_key_type! { struct PropKey; }
new_key_type! { struct TaskKey; }

/// State
struct State {
    task_map: HashMap<TaskID, TaskKey>,
	tasks: SlotMap<TaskKey, TaskShort>,
	/// map property names to slotmap ids
	prop_map: HashMap<(TaskID, String), PropKey>,
	/// efficient, dense storage of all locally-stored task properties
	props: SlotMap<PropKey, TaskPropVariant>,
	/// scripts
	scripts: HashMap<ScriptID, Script>,
	/// view data
	views: HashMap<ViewID, View>,
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
