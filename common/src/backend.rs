//! This file outlines all the structures required for the middleware and backend to communicate via REST API

use serde::{Deserialize, Serialize};

use crate::*;

/// # TASK API
/// reawest::get("/task")
#[derive(Serialize, Deserialize, Debug)]
pub struct ReadTaskShortRequest {
    /// task id to request
    pub task_id: TaskID,
    /// id of request
    pub req_id: u64,
}
/// response to GET /task
#[derive(Serialize, Deserialize, Debug, Default)]
pub struct ReadTaskShortResponse {
    /// task id of response, should be the same as request
    pub task_id: TaskID,
    /// name of task
    pub name: String,
    /// completion status of task
    pub completed: bool,
    /// list of string names of properties
    pub props: Vec<String>,
    /// list of task ids that are dependants
    pub deps: Vec<TaskID>,
    /// list of script ids that apply to this task
    pub scripts: Vec<ScriptID>,
    /// last time this task was edited
    pub last_edited: chrono::NaiveDateTime,
    /// id of request
    pub req_id: u64,
}
/// request to GET /tasks, just list of GET /task requests
pub type ReadTasksShortRequest = Vec<ReadTaskShortRequest>;
/// response to GET /tasks, just list of GET /task responses
pub type ReadTasksShortResponse = Vec<Result<ReadTaskShortResponse, String>>;

/// reqwest::post("/task").body(CreateTaskRequest {})
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct CreateTaskRequest {
    /// name of task
    pub name: String,
    /// completion status of task
    pub completed: bool,
    /// list of properties to add to task
    pub properties: Vec<TaskProp>,
    /// [name, date, value]
    pub dependencies: Vec<TaskID>,
    /// id of request
    pub req_id: i32,
}
/// response to POST /task contains the ID of the created task.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct CreateTaskResponse {
    /// id of task
    pub task_id: TaskID,
    /// id of request
    pub req_id: i32,
}

/// reqwest::post("/tasks").body(CreateTaskRequest {})
pub type CreateTasksRequest = Vec<CreateTaskRequest>;
/// a list of task ids that were created
pub type CreateTasksResponse = Vec<CreateTaskResponse>;

/// reqwest::put("/task")
#[derive(Serialize, Deserialize)]
pub struct UpdateTaskRequest {
    /// task id
    pub task_id: TaskID,
    /// name change
    pub name: Option<String>,
    /// checked change
    pub checked: Option<bool>,
    /// props to add
    pub props_to_add: Vec<TaskProp>,
    /// props to remove
    pub props_to_remove: Vec<String>,
    /// deps to add
    pub deps_to_add: Vec<TaskID>,
    /// deps to remove
    pub deps_to_remove: Vec<TaskID>,
    /// scripts to add
    pub scripts_to_add: Vec<ScriptID>,
    /// scripts to remove
    pub scripts_to_remove: Vec<ScriptID>,
    /// id of request
    pub req_id: i32,
}
/// respone is just taskid
#[derive(Serialize, Deserialize)]
pub struct UpdateTaskResponse {
    /// id of task
    pub task_id: TaskID,
    /// id of request
    pub req_id: u64,
}
/// reqwest::put("/tasks")
pub type UpdateTasksRequest = Vec<UpdateTaskRequest>;
/// response is just taskids
pub type UpdateTasksResponse = Vec<UpdateTaskResponse>;
/// reqwest::delete("/task")
#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteTaskRequest {
    /// id to delete
    pub task_id: TaskID,
    /// id of request
    pub req_id: i32,
}
/// response is empty
pub type DeleteTaskResponse = TaskID;
/// reawest::delete("/tasks")
pub type DeleteTasksRequest = Vec<DeleteTaskRequest>;
/// response is empty
pub type DeleteTasksResponse = Vec<i32>;

/// # PROPERTIES API

/// reqwest::get("/prop")
#[derive(Serialize, Deserialize)]
pub struct PropertyRequest {
    /// task id
    pub task_id: TaskID,
    /// list of property names we want to get values for
    pub properties: Vec<String>,
    /// id of request
    pub req_id: u64,
}
/// response to GET /props
#[derive(Serialize, Deserialize)]
pub struct PropertyResponse {
    /// actual result
    pub res: Vec<TaskPropOption>,
    /// id of request
    pub req_id: u64,
}
/// individual property but an option
#[derive(Serialize, Deserialize)]
pub struct TaskPropOption {
    /// name of property
    pub name: String,
    /// value of property
    pub value: Option<TaskPropVariant>,
}
/// reqwest::get("/props")
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct PropertiesRequest {
    /// list of task ids we want properties for
    pub task_ids: Vec<TaskID>,
    /// list of properties we want to get for each
    pub properties: Vec<String>,
    /// id of request
    pub req_id: u64,
}
/// does smth
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct PropertiesResponse {
    /// actual result
    pub res: Vec<TaskPropColumn>,
    /// id of request
    pub req_id: u64,
}
/// column of task properties with name
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct TaskPropColumn {
    /// name of property
    pub name: String,
    /// properties ordered by the taskid they were requested in
    pub values: Vec<Option<TaskPropVariant>>,
}

/// # FILTER APIS

/// reqwest::get("/filter")
#[derive(Serialize, Deserialize, Debug)]
pub struct FilterRequest {
    /// filter to apply
    pub filter: Filter,
}
/// responose to GET /filter
pub type FilterResponse = Vec<TaskID>;
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
            req_id: 0,
        });
    }

    #[test]
    fn serde_properties_request() {
        test_serde_commutes(PropertiesRequest {
            task_ids: vec![1],
            properties: vec!["hi".to_string()],
            req_id: 0,
        })
    }
    #[test]
    fn serde_properties_response() {
        test_serde_commutes(PropertiesResponse {
            req_id: 0,
            res: vec![TaskPropColumn {
                name: "dog".to_string(),
                values: vec![None, Some(TaskPropVariant::String("dog2".to_string()))],
            }],
        })
    }
}
