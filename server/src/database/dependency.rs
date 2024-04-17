use sea_orm::entity::prelude::*;
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "dependency")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub task_id: i32,
    #[sea_orm(primary_key)]
    pub depends_on_id: i32,
}
#[derive(Copy, Clone, Debug, EnumIter, PartialEq, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::task::Entity",
        from = "Column::TaskId",
        to = "super::task::Column::Id"
    )]
    Task,
}
impl Related<super::task::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Task.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}

pub struct DependencyLink;

impl Linked for DependencyLink {
    type FromEntity = super::dependency::Entity;
    type ToEntity = super::task::Entity;
    fn link(&self) -> Vec<RelationDef> {
        vec![
            super::task::Relation::Dependency.def().rev(),
            super::task::Entity::belongs_to(super::dependency::Entity)
                .from(super::task::Column::Id)
                .to(super::dependency::Column::DependsOnId)
                .into(),
        ]
    }
}
#[cfg(test)]
mod tests {
    use sea_orm::{DatabaseBackend, Iterable, MockDatabase};

    use crate::database::dependency;
    use crate::database::task;

    use super::*;

    #[test]
    fn test_copy_clone_debug() {
        let original = Relation::Task;
        let copy = original;
        assert_eq!(original, copy);
        let clone = original.clone();
        assert_eq!(original, clone);
        format!("{:?}", original);
    }

    #[test]
    fn test_enum_iter() {
        let mut iter = Relation::iter();
        assert_eq!(iter.next(), Some(Relation::Task));
        assert_eq!(iter.next(), None);
    }
    #[tokio::test]
    async fn test_dependency_relations() {
        let db = MockDatabase::new(DatabaseBackend::Postgres);
        let created_time = chrono::offset::Utc::now().naive_utc();
        let task_owned_by_dependency = task::Model {
            id: 1,
            title: "taskOwnedByDependency".to_owned(),
            completed: false,
            last_edited: created_time,
        };
        let task_depending_on_another_task = task::Model {
            id: 2,
            title: "taskDependingOnAnotherTask".to_owned(),
            completed: false,
            last_edited: created_time,
        };
        let dependency_row = dependency::Model {
            task_id: 2,
            depends_on_id: 1,
        };
        let db_conn = db
            .append_query_results([[(task_owned_by_dependency, dependency_row)]])
            .into_connection();
        let query_result = task::Entity::find_by_id(1)
            .find_also_linked(task::TaskOwnedLink)
            .all(&db_conn)
            .await
            .unwrap();
        println!("{:?}", query_result);
    }
}
