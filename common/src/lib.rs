#![warn(rustdoc::private_doc_tests)]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![allow(dead_code)] // this is for
//! this crate provides the common types and traits for interacting
//! with the backend and middleware
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

/// Type alias for property ID
pub type PropID = String;

/// A view is list of filtered tasks
pub struct View {
    filter: Filter,
    props: Vec<String>,
    tasks: Vec<TaskID>,
}

/// A script is a piece of code that can be run to modify tasks
pub struct Script {
    content: String,
}

/// Types of Comparators for filters
#[derive(Serialize, Deserialize)]
pub enum Comparator {
    /// Less than
    LT,
    /// Less than or equal to
    LEQ,
    /// Greater than
    GT,
    /// Greater than or equal to
    GEQ,
    /// Equal to
    EQ,
    /// Not equal to
    NEQ,
    /// Contains
    CONTAINS,
    /// Does not contain
    NOTCONTAINS,
    /// Regular expression match
    REGEX,
}

/// Types of logical operators for filters
#[derive(Serialize, Deserialize)]
pub enum Operator {
    /// Logical AND
    AND,
    /// Logical OR
    OR,
}
/// the types of properties that can be stored for tasks
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TaskPropVariant {
    /// Date variant
    Date(chrono::DateTime<chrono::Local>),
    /// String variant
    String(String),
    /// Number variant
    Number(f64),
    /// Boolean variant
    Boolean(bool),
}
/// A property of a task is a name-value pair
#[derive(Serialize, Deserialize)]
pub struct TaskProp {
    name: String,
    value: TaskPropVariant,
}

/// A filter is a tree of comparators and operators that can be used to filter tasks
#[derive(Serialize, Deserialize)]
pub enum Filter {
    /// A leaf node in the filter tree
    Leaf {
        /// the comparator to apply to the field
        comparator: Comparator,
        /// the field to compare
        field: TaskProp,
        /// the value to compare to
        immediate: TaskPropVariant,
    },
    /// A node in the filter tree with children cannot be a leaf
    Operator {
        /// the operator to apply to the children
        op: Operator,
        /// the children of this node they are also filters
        childs: Vec<Filter>,
    },
}
