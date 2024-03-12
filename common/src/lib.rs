//! The common crate contains shared structures and functions in use by the client/middleware and server implementations.

#![warn(rustdoc::private_doc_tests)]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![allow(unused)]

pub mod backend;

use serde::{Deserialize, Serialize};
/// Primary key for tasks
/// Note: Database should ensure IDs are never re-used.
pub type TaskID = u64;

/// Primary key for scripts
/// Note: Database should ensure IDs are never re-used.
pub type ScriptID = u64;

/// Primary key for views
/// Note: Database should ensure IDs are never re-used.
pub type ViewID = u64;

/// Identification of a property, from database
pub type PropName = String;
/// A view is list of filtered tasks
pub struct View {
    filter: Filter,
    props: Vec<String>,
    tasks: Vec<TaskID>,
}

/// Primary Task Data (doesn't include properties)
pub struct TaskShort {
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

/// The content of a lua script
pub struct Script {
    content: String,
}

/// Types of Comparators for filters
#[derive(Serialize, Deserialize)]
#[allow(missing_docs)]
pub enum Comparator {
    LT,
    LEQ,
    GT,
    GEQ,
    EQ,
    NEQ,
    CONTAINS,
    NOTCONTAINS,
    REGEX,
}
#[derive(Serialize, Deserialize)]
pub enum Operator {
    AND,
    OR,
}

/// The variants of Task Properties
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TaskPropVariant {
    Date(()),
    String(String),
    Number(f64),
    Boolean(bool),
}
/// A task property and its corresponding name.
#[derive(Serialize, Deserialize)]
pub struct TaskProp {
    name: String,
    value: TaskPropVariant,
}

/// Represents a filter on tasks applied to the database serverside.
#[derive(Serialize, Deserialize)]
pub enum Filter {
    Leaf {
        comparator: Comparator,
        field: TaskProp,
        immediate: TaskPropVariant,
    },
    Operator {
        op: Operator,
        childs: Vec<Filter>,
    },
}
