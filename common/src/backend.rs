//! This file outlines all the structures required for the middleware and backend to communicate via REST API

use std::{fs::read, path::Path};
use std::fs;
use serde::{Deserialize, Serialize};

use crate::*;
use actix_settings::BasicSettings;

/// Default toml template for server / client settings
pub const ABN_DEFAULT_TOML_TEMPLATE: & str = "[actix]\n# For more info, see: https://docs.rs/actix-web/4/actix_web/struct.HttpServer.html.\n\nhosts = [\n    [\"0.0.0.0\", 8080]      # This should work for both development and deployment...\n    #                      # ... but other entries are possible, as well.\n]\nmode = \"development\"       # Either \"development\" or \"production\".\nenable-compression = true  # Toggle compression middleware.\nenable-log = true          # Toggle logging middleware.\n\n# The number of workers that the server should start.\n# By default the number of available logical cpu cores is used.\n# Takes a string value: Either \"default\", or an integer N > 0 e.g. \"6\".\nnum-workers = \"default\"\n\n# The maximum number of pending connections. This refers to the number of clients\n# that can be waiting to be served. Exceeding this number results in the client\n# getting an error when attempting to connect. It should only affect servers under\n# significant load. Generally set in the 64-2048 range. The default value is 2048.\n# Takes a string value: Either \"default\", or an integer N > 0 e.g. \"6\".\nbacklog = \"default\"\n\n# Sets the per-worker maximum number of concurrent connections. All socket listeners\n# will stop accepting connections when this limit is reached for each worker.\n# By default max connections is set to a 25k.\n# Takes a string value: Either \"default\", or an integer N > 0 e.g. \"6\".\nmax-connections = \"default\"\n\n# Sets the per-worker maximum concurrent connection establish process. All listeners\n# will stop accepting connections when this limit is reached. It can be used to limit\n# the global TLS CPU usage. By default max connections is set to a 256.\n# Takes a string value: Either \"default\", or an integer N > 0 e.g. \"6\".\nmax-connection-rate = \"default\"\n\n# Set server keep-alive preference. By default keep alive is set to 5 seconds.\n# Takes a string value: Either \"default\", \"disabled\", \"os\",\n# or a string of the format \"N seconds\" where N is an integer > 0 e.g. \"6 seconds\".\nkeep-alive = \"default\"\n\n# Set server client timeout in milliseconds for first request. Defines a timeout\n# for reading client request header. If a client does not transmit the entire set of\n# headers within this time, the request is terminated with the 408 (Request Time-out)\n# error. To disable timeout, set the value to 0.\n# By default client timeout is set to 5000 milliseconds.\n# Takes a string value: Either \"default\", or a string of the format \"N milliseconds\"\n# where N is an integer > 0 e.g. \"6 milliseconds\".\nclient-timeout = \"default\"\n\n# Set server connection shutdown timeout in milliseconds. Defines a timeout for\n# shutdown connection. If a shutdown procedure does not complete within this time,\n# the request is dropped. To disable timeout set value to 0.\n# By default client timeout is set to 5000 milliseconds.\n# Takes a string value: Either \"default\", or a string of the format \"N milliseconds\"\n# where N is an integer > 0 e.g. \"6 milliseconds\".\nclient-shutdown = \"default\"\n\n# Timeout for graceful workers shutdown. After receiving a stop signal, workers have\n# this much time to finish serving requests. Workers still alive after the timeout\n# are force dropped. By default shutdown timeout sets to 30 seconds.\n# Takes a string value: Either \"default\", or a string of the format \"N seconds\"\n# where N is an integer > 0 e.g. \"6 seconds\".\nshutdown-timeout = \"default\"\n\n[actix.tls] # TLS is disabled by default because the certs don't exist\nenabled = false\ncertificate = \"path/to/cert/cert.pem\"\nprivate-key = \"path/to/cert/key.pem\"\n\n# The `application` table be used to express application-specific settings.\n# See the `README.md` file for more details on how to use this.\n[application]\ndatabase_url = \"postgres://abn:abn@localhost:5432/abn?options=-c%20search_path%3Dtask\"";

/// Struct for storing application-specific settings for ABN
#[derive(Debug, Deserialize)]
pub struct DatabaseSettings {
    /// The url of the database
    pub database_url: String,
}

/// ABN-Specific BasicSettings Type 
pub type AbnSettings = BasicSettings<DatabaseSettings>;

/// Function called by client / server main to load settings
pub fn load_settings() -> Result<AbnSettings, actix_settings::Error> {
    // if Server.toml does not exist in working directory, make a new one
    let settings_filepath = Path::new(".abn_settings").join("Settings.toml");
    match fs::metadata(&settings_filepath) {
        Ok(_) => {
            AbnSettings::parse_toml(&settings_filepath)
        },
        Err(_) => {
            println!("creating directory");
            fs::create_dir(Path::new(".abn_settings")).unwrap();
            AbnSettings::write_toml_file(&settings_filepath).unwrap();
            fs::write(&settings_filepath, backend::ABN_DEFAULT_TOML_TEMPLATE).unwrap();
            AbnSettings::parse_toml(&settings_filepath)
        },
    }
}

/// # TASK API
/// reawest::get("/task")
#[derive(Debug, Serialize, Deserialize)]
pub struct ReadTaskShortRequest {
    /// task id to request
    pub task_id: TaskID,
    /// id of request
    pub req_id: u64,
}
/// response to GET /task
#[derive(Debug, Serialize, Deserialize, Default)]
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
    /// id of request
    pub req_id: u64,
}
/// response to POST /task contains the ID of the created task.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct CreateTaskResponse {
    /// id of task
    pub task_id: TaskID,
    /// id of request
    pub req_id: u64,
}

/// reqwest::post("/tasks").body(CreateTaskRequest {})
pub type CreateTasksRequest = Vec<CreateTaskRequest>;
/// a list of task ids that were created
pub type CreateTasksResponse = Vec<CreateTaskResponse>;

/// reqwest::put("/task")
#[derive(Debug, Serialize, Deserialize)]
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
    pub req_id: u64,
}
/// respone is just taskid
#[derive(Debug, Serialize, Deserialize)]
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
#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteTaskRequest {
    /// id to delete
    pub task_id: TaskID,
    /// client-side id of request (encoded TaskKey)
    pub req_id: u64,
}
/// response encodes client-side TaskKey to delete
pub type DeleteTaskResponse = u64;
/// reawest::delete("/tasks")
pub type DeleteTasksRequest = Vec<DeleteTaskRequest>;
/// response encodes list of client-side TaskKeys to delete
pub type DeleteTasksResponse = Vec<u64>;

/// # PROPERTIES API

/// reqwest::get("/prop")
#[derive(Debug, Serialize, Deserialize)]
pub struct PropertyRequest {
    /// task id
    pub task_id: TaskID,
    /// list of property names we want to get values for
    pub properties: Vec<String>,
    /// id of request
    pub req_id: u64,
}
/// response to GET /props
#[derive(Debug, Serialize, Deserialize)]
pub struct PropertyResponse {
    /// actual result
    pub res: Vec<TaskPropOption>,
    /// id of request
    pub req_id: u64,
}
/// individual property but an option
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskPropOption {
    /// name of property
    pub name: String,
    /// value of property
    pub value: Option<TaskPropVariant>,
}
/// reqwest::get("/props")
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PropertiesRequest {
    /// list of task ids we want properties for
    pub task_ids: Vec<TaskID>,
    /// list of properties we want to get for each
    pub properties: Vec<String>,
    /// id of request
    pub req_id: u64,
}
/// does smth
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PropertiesResponse {
    /// actual result
    pub res: Vec<TaskPropColumn>,
    /// id of request
    pub req_id: u64,
}
/// column of task properties with name
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct TaskPropColumn {
    /// name of property
    pub name: String,
    /// properties ordered by the taskid they were requested in
    pub values: Vec<Option<TaskPropVariant>>,
}

/// # FILTER APIS

/// reqwest::get("/filter")
#[derive(Debug, Serialize, Deserialize)]
pub struct FilterRequest {
    /// filter to apply
    pub filter: Filter,
    /// request ID
    pub req_id: u64,
}
/// responose to GET /filter
#[derive(Debug, Serialize, Deserialize)]
pub struct FilterResponse {
    /// list of task ids that match the filter
    pub tasks: Vec<TaskID>,
    /// request id for middleware
    pub req_id: u64,
}
/// reqwest::get("/filter")
struct FilterTaskRequest {
    filter: Filter,
    props: Vec<String>,
}
type FilterTaskRespone = Vec<TaskShort>;

/// request for GET /views
pub type GetViewRequest = u64;
/// response for GET /views
#[derive(Debug, Serialize, Deserialize)]
pub struct GetViewResponse {
    /// the views to be reutned
    pub views: Vec<ViewData>,
    /// the request id
    pub req_id: u64,
}
/// request for POST /view
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateViewRequest {
    /// name of view
    pub name: String,
    /// props you want to display
    pub props: Vec<String>,
    /// filter for view
    pub filter: Filter,
    /// the request id
    pub req_id: u64,
}
/// response for POST /view
#[derive(Debug, Serialize, Deserialize)]
pub struct CreateViewResponse {
    /// ID of view
    pub view_id: i32,
    /// ID of request
    pub req_id: u64,
}
/// request for PUT /view
#[derive(Debug, Serialize, Deserialize)]
pub struct UpdateViewRequest {
    /// new view that we're setting
    pub view: ViewData,
    /// ID of request
    pub req_id: u64,
}
/// response for PUT /view
pub type UpdateViewResponse = u64;
/// request for DELETE /view
pub type DeleteViewRequest = i32;
/// response for DELETE /view
pub type DeleteViewResponse = ();

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
