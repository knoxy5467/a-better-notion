//! The common crate contains shared structures and functions in use by the client/middleware and server implementations.

#![warn(rustdoc::private_doc_tests)]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]

#[allow(dead_code)]

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

/// The content of a lua script.
/// Scripts are used to modify tasks based on events.
pub struct Script {
    content: String,
}

/// Types of Comparators for filters
#[derive(Serialize, Deserialize)]
#[allow(missing_docs)]
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

/// Operator that combines multiple Filters
#[derive(Serialize, Deserialize)]
pub enum Operator {
    /// AND operator, takes the intersection of the results of a set of filters.
    AND,
    /// OR operator, appends the results of all the filters to each other.
    OR,
}

/// The variants of Task Properties
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TaskPropVariant {
    /// Local-time-zone representation of postgresql's timestamp
    Date(chrono::DateTime<chrono::Local>),
    /// String variant
    String(String),
    /// Decimal variant (NOTE: should we have an integer variant?)
    Number(f64),
    /// Boolean variant
    Boolean(bool),
}
/// A task property and its corresponding name.
#[derive(Serialize, Deserialize)]
pub struct TaskProp {
    name: String,
    value: TaskPropVariant,
}

/// Represents a filter on tasks using their properties that the database computes.
#[derive(Serialize, Deserialize)]
pub enum Filter {
    /// Filter leaf, represents a comparator that filters properties
    Leaf {
        /// Property name to filter on.
        field: TaskProp,
        /// Method by which a task's property is compared to `immediate` to determine if
        /// property should be filtered out or not.
        comparator: Comparator,
        /// Immediate value to use in the comparison
        immediate: TaskPropVariant,
    },
    /// Filter branch, combines multiple leaves based on Operator.
    Operator {
        /// operator used to combined a set of nested filters
        op: Operator,
        /// the nested filters to be combined
        childs: Vec<Filter>,
    },
}
