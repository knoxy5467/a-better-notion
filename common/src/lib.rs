//! The common crate contains shared structures and functions in use by the client/middleware and server implementations.

#![warn(rustdoc::private_doc_tests)]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
#![allow(unused)]

pub mod backend;

use serde::{Deserialize, Serialize};

/// Database Primary key for tasks
/// Note: Database should ensure IDs are never re-used.
pub type TaskID = u64;

/// Database Primary key for scripts
/// Note: Database should ensure IDs are never re-used.
pub type ScriptID = u64;

/// Database Primary key for views
/// Note: Database should ensure IDs are never re-used.
pub type ViewID = u64;

/// Identification of a property, from database
pub type PropName = String;
/// A view is list of filtered tasks
#[derive(Debug, Default)]
pub struct View {
    filter: Filter,
    props: Vec<String>,
    tasks: Vec<TaskID>,
}

/// Primary Task Data (doesn't include properties)
#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
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
#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Script {
    content: String,
}

/// Types of Comparators for filters
#[derive(Debug, Serialize, Deserialize, PartialEq)]
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
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Operator {
    /// AND operator, takes the intersection of the results of a set of filters.
    AND,
    /// OR operator, appends the results of all the filters to each other.
    OR,
}

/// The variants of Task Properties
/// Note: serialization with serde(tag = "...") doesn't work for tuple enums.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum TaskPropVariant {
    /// Local-time-zone representation of postgresql's timestamp
    Date(chrono::DateTime<chrono::Utc>),
    /// String variant
    String(String),
    /// Decimal variant (NOTE: should we have an integer variant?)
    Number(f64),
    /// Boolean variant
    Boolean(bool),
}
/// A task property and its corresponding name.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct TaskProp {
    name: String,
    value: TaskPropVariant,
}

/// Represents a filter on tasks using their properties that the database computes.
#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub enum Filter {
    /// Filter leaf, represents a comparator that filters properties
    Leaf {
        /// Property name to filter on.
        field: String,
        /// Method by which a task's property is compared to `immediate` to determine if
        /// property should be filtered out or not.
        comparator: Comparator,
        /// Immediate value to use with the comparator
        immediate: TaskPropVariant,
    },
    /// Filter branch, combines multiple leaves based on Operator.
    Operator {
        /// operator used to combined a set of nested filters
        op: Operator,
        /// the nested filters
        childs: Vec<Filter>,
    },
    #[default]
    /// "Null" Filter (so we can implement Default)
    None,
}

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
    fn serde_task_prop() {
        test_serde_commutes(TaskProp {
            name: "test".to_owned(),
            value: TaskPropVariant::Date(chrono::Utc::now()),
        });
    }
    #[test]
    fn serde_filter() {
        test_serde_commutes(Filter::None);
        test_serde_commutes(Filter::Leaf {
            field: "test".to_owned(),
            comparator: Comparator::EQ,
            immediate: TaskPropVariant::Boolean(true),
        });
        test_serde_commutes(Filter::Operator {
            op: Operator::AND,
            childs: vec![],
        });
    }

    #[test]
    fn test_view() {
        dbg!(View::default());
    }

    #[test]
    fn serde_task_short() {
        test_serde_commutes(TaskShort::default());
    }

    #[test]
    fn serde_script() {
        test_serde_commutes(Script::default());
    }
}
