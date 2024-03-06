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
struct ReadTaskShortData {
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

/// reqwest::put("/task")
struct UpdateTaskData {
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
type UpdateTasksData = Vec<UpdateTaskData>;
type UpdateTasksResponse = Vec<TaskID>;
/// reqwest::delete("/task")
struct DeleteTaskData {
	task_id: TaskID
}
type DeleteTaskResponse = ();
/// reawest::delete("/tasks")
type DeleteTasksData = Vec<DeleteTaskData>;
type DeleteTasksResponse = ();

////// FILTER STUFF
enum Comparator { LT, LEQ, GT, GEQ, EQ, NEQ, CONTAINS, NOTCONTAINS, REGEX }
enum Operator { AND, OR }
enum Filter {
	Leaf { comparator: Comparator, field: TaskProp, immediate: TaskPropVariant },
	Operator { op: Operator, childs: Vec<Filter> },
}

/// reqwest::get("/filterid")
struct FilterTaskIDsData {
	filter: Filter
}
type FilterTaskIDsResponse = Vec<TaskID>;
/// reqwest::get("/filter")
struct FilterTaskResponse {
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

trait API {
	// API stuff here
}

enum Value {
	String(String),
	Decimal(f64),
	Number(i64),
}
struct Prop {
	name: String,
	item: Value,
}
struct Task {
	props: Vec<Prop>,
}
struct State {
	tasks: Vec<Task>
}