//! This file outlines all the structures required for the middleware and backend to communicate via REST API

use serde::{Deserialize, Serialize};

use crate::*;

/// # TASK API

/// reqwest::post("/task").body(CreateTaskRequest {})    
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct CreateTaskRequest {
    name: String,
    completed: bool,
    properties: Vec<TaskProp>,
    /// [name, date, value]
    dependencies: Vec<TaskID>,
}
type CreateTaskResponse = TaskID;
/// reqwest::post("/tasks").body(CreateTaskRequest {})
type CreateTasksRequest = Vec<CreateTaskRequest>;
type CreateTasksResponse = Vec<TaskID>;

/// reawest::get("/task")
#[derive(Serialize, Deserialize)]
pub struct ReadTaskShortRequest {
    // task id to request
    pub task_id: TaskID,
}
// response to GET /task
#[derive(Serialize, Deserialize)]
pub struct ReadTaskShortResponse {
    // task id of response, should be the same as request
    pub task_id: TaskID,
    // name of task
    pub name: String,
    // completion status of task
    pub completed: bool,
    // list of string names of properties
    pub props: Vec<String>,
    // list of task ids that are dependants
    pub deps: Vec<TaskID>,
    // list of script ids that apply to this task
    pub scripts: Vec<ScriptID>,
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
    scripts_to_remove: Vec<ScriptID>,
}
type UpdateTaskResponse = TaskID;
/// reqwest::put("/tasks")
type UpdateTasksRequest = Vec<UpdateTaskRequest>;
type UpdateTasksResponse = Vec<TaskID>;
/// reqwest::delete("/task")
struct DeleteTaskRequest {
    task_id: TaskID,
}
type DeleteTaskResponse = ();
/// reawest::delete("/tasks")
type DeleteTasksRequest = Vec<DeleteTaskRequest>;
type DeleteTasksResponse = ();

/// # PROPERTIES API

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

/// # FILTER APIS

/// reqwest::get("/filterid")
struct FilterTaskIDsRequest {
    filter: Filter,
}
type FilterTaskIDsResponse = Vec<TaskID>;
/// reqwest::get("/filter")
struct FilterTaskRequest {
    filter: Filter,
    props: Vec<String>,
}
type FilterTaskRespone = Vec<TaskShort>;

#[cfg(test)]
mod tests {
    use chrono::DateTime;

    use super::*;

    fn test_serde_commutes<T: std::fmt::Debug + Serialize + for<'a> Deserialize<'a> + PartialEq>(
        obj: T,
    ) {
        let serialized = serde_json::to_string(&obj).unwrap();
        let deser_obj = serde_json::from_str(&serialized).unwrap();
        assert_eq!(obj, deser_obj);
    }

    #[test]
    fn serde_create_task_request() {
        test_serde_commutes(CreateTaskRequest {
            name: "test".to_owned(),
            completed: false,
            properties: vec![],
            dependencies: vec![],
        });
    }
}
