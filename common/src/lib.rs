#![warn(rustdoc::private_doc_tests)]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
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

pub type PropID = String;
/// A view is list of filtered tasks
pub struct View {
    filter: Filter,
    props: Vec<String>,
    tasks: Vec<TaskID>,
}

pub struct Script {
    content: String,
}

/// Types of Comparators for filters
#[derive(Serialize, Deserialize)]
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
#[derive(Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TaskPropVariant {
    Date(()),
    String(String),
    Number(f64),
    Boolean(bool),
}
#[derive(Serialize, Deserialize)]
pub struct TaskProp {
    name: String,
    value: TaskPropVariant,
}
/// A filter is a tree of comparators and operators that can be used to filter tasks
#[derive(Serialize, Deserialize)]
pub enum Filter {
    //! A leaf node in the filter tree
    Leaf {
        comparator: Comparator,
        field: TaskProp,
        immediate: TaskPropVariant,
    },
    //! A node in the filter tree with children cannot be a leaf
    Operator {
        op: Operator,
        childs: Vec<Filter>,
    },
}
