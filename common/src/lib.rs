#![warn(rustdoc::private_doc_tests)]
#![warn(missing_docs)]
#![warn(rustdoc::missing_crate_level_docs)]
/// Primary key for tasks
/// Note: Database should ensure IDs are never re-used.
type TaskID = u64;

/// Primary key for scripts
/// Note: Database should ensure IDs are never re-used.
type ScriptID = u64;

/// Primary key for views
/// Note: Database should ensure IDs are never re-used.
type ViewID = u64;
/// A view is list of filtered tasks
struct View {
    filter: Filter,
    props: Vec<String>,
    tasks: Vec<TaskID>,
}

struct Script {
    content: String,
}

/// Types of Comparators for filters
#[derive(Serialize, Deserialize)]
enum Comparator {
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
enum Operator {
    AND,
    OR,
}
#[derive(Serialize, Deserialize)]
enum Filter {
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
