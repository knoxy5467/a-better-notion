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
