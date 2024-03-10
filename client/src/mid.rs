//! Middleware Structs

#[derive(Serilize, Deserialize)]
#[serde(tag="type")]
enum TaskPropVariant {
	Date(()),
	String(String),
	Number(f64),
	Boolean(bool),
}
#[serde(Serialize, Deserialize)]
struct TaskProp {
	name: String,
	value: TaskPropVariant,
}

type ScriptID = u64;
slotmap::new_key_type! { struct TaskID; }
struct Task {
	task_id: TaskID,
	name: String,
	completed: bool,
	properties: SlotHashMap<String, TaskPropVariant>,
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
#[derive(Serilize, Deserialize)]
struct CreateTaskRequest {
	name: String,
	completed: bool,
	properties: Vec<TaskProp>,
	/// [name, date, value]
	dependencies: Vec<TaskID>
}
type CreateTaskResponse = TaskID;
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
	scripts: Vec<ScriptID>
}
type ReadTasksShortRequest = Vec<ReadTaskShortRequest>;
type ReadTasksShortResponse = Vec<ReadTaskShortResponse>;

/// reqwest::put("/task")
struct UpdateTaskRequest {
	task_id: TaskID,
	name: Option<String>,
	checked: Option<bool>,
	props_to_add: Vec<TaskProp>,
	props_to_remove: Vec<String>,
	deps_to_add: Vec<TaskID>,
	deps_to_remove: Vec<TaskID>,
	scripts_to_add: Vec<ScriptID>,
	scripts_to_remove: Vec<ScriptID>
}
type UpdateTaskResponse = TaskID;
/// reqwest::put("/tasks")
type UpdateTasksRequest = Vec<UpdateTaskRequest>;
type UpdateTasksResponse = Vec<TaskID>;
/// reqwest::delete("/task")
struct DeleteTaskRequest {
	task_id: TaskID
}
type DeleteTaskResponse = ();
/// reawest::delete("/tasks")
type DeleteTasksRequest = Vec<DeleteTaskRequest>;
type DeleteTasksResponse = ();

///// PROPERTIES STUFF
/// reqwest::get("/prop")
struct PropertyRequest {
	task_id: TaskID,
	properties: Vec<String>,
}
type PropertyResponse = Vec<(String, TaskPropVariant)>;
/// reqwest::get("/props")
struct PropertiesRequest {
	task_id: Vec<TaskID>,
	properties: Vec<String>,
}
type PropertiesResponse = Vec<(String, Vec<TaskPropVariant>)>;

////// FILTER STUFF
enum Comparator { LT, LEQ, GT, GEQ, EQ, NEQ, CONTAINS, NOTCONTAINS, REGEX }
enum Operator { AND, OR }
enum Filter {
	Leaf { comparator: Comparator, field: TaskProp, immediate: TaskPropVariant },
	Operator { op: Operator, childs: Vec<Filter> },
}

/// reqwest::get("/filterid")
struct FilterTaskIDsRequest {
	filter: Filter
}
type FilterTaskIDsResponse = Vec<TaskID>;
/// reqwest::get("/filter")
struct FilterTaskRequest {
	filter: Filter,
	props: Vec<String>
}
type FilterTaskRespone = Vec<Task>;


/// A view is a reference
struct View {
	filter: Filter,
	props: Vec<String>,
	max_tasks: Option<u64>,
}
