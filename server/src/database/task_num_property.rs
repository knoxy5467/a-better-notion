use sea_orm::entity::prelude::*;

use crate::database::{task_num_property, task_property};

#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "task_num_property")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub task_id: i32,
    #[sea_orm(primary_key)]
    pub name: String,
    pub value: Decimal,
}

#[derive(Copy, Clone, Debug, EnumIter, PartialEq, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::task::Entity",
        from = "Column::TaskId",
        to = "super::task::Column::Id"
    )]
    Task,
    #[sea_orm(
        belongs_to = "super::task_property::Entity",
        from = "Column::Name",
        to = "super::task_property::Column::Name"
    )]
    TaskProperty,
}
impl Related<super::task::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Task.def()
    }
}
impl Related<super::task_property::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TaskProperty.def()
    }
}
impl ActiveModelBehavior for ActiveModel {}

pub struct TaskPropertyLink;

impl Linked for TaskPropertyLink {
    type FromEntity = super::task_property::Entity;
    type ToEntity = super::task_num_property::Entity;

    fn link(&self) -> Vec<RelationDef> {
        vec![
            task_num_property::Entity::belongs_to(task_property::Entity)
                .from(task_num_property::Column::TaskId)
                .to(task_property::Column::TaskId)
                .into(),
            task_num_property::Entity::belongs_to(task_property::Entity)
                .from(task_num_property::Column::Name)
                .to(task_property::Column::Name)
                .into(),
            task_property::Entity::has_one(task_num_property::Entity)
                .from(task_property::Column::TaskId)
                .to(task_num_property::Column::TaskId)
                .into(),
            task_property::Entity::has_one(task_num_property::Entity)
                .from(task_property::Column::Name)
                .to(task_num_property::Column::Name)
                .into(),
        ]
    }
}
#[cfg(test)]
mod tests {
    use sea_orm::Iterable;

    use super::*;
    use sea_orm::{DatabaseBackend, MockDatabase};

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
        assert_eq!(iter.next(), Some(Relation::TaskProperty));
        assert_eq!(iter.next(), None);
    }
    #[tokio::test]
    async fn test_task_num_related() {
        let db = MockDatabase::new(DatabaseBackend::Postgres);
        let db_connection = db
            .append_query_results([[(
                task_num_property::Model {
                    task_id: 1,
                    name: "test".to_owned(),
                    value: rust_decimal::Decimal::new(1, 0),
                },
                crate::database::task_property::Model {
                    task_id: 1,
                    name: "test".to_owned(),
                    typ: "num".to_owned(),
                },
            )]])
            .into_connection();

        assert_eq!(
            crate::database::task_num_property::Entity::find()
                .find_also_related(crate::database::task_property::Entity)
                .all(&db_connection)
                .await
                .unwrap(),
            [(
                task_num_property::Model {
                    task_id: 1,
                    name: "test".to_owned(),
                    value: rust_decimal::Decimal::new(1, 0),
                },
                Some(crate::database::task_property::Model {
                    task_id: 1,
                    name: "test".to_owned(),
                    typ: "num".to_owned(),
                })
            )]
        )
    }
    #[tokio::test]
    async fn test_task_num_property_find_also_linked() {
        let db = MockDatabase::new(DatabaseBackend::Postgres);
        let db_connection = db
            .append_query_results([[(
                crate::database::task_property::Model {
                    task_id: 1,
                    name: "test".to_owned(),
                    typ: "num".to_owned(),
                },
                task_num_property::Model {
                    task_id: 1,
                    name: "test".to_owned(),
                    value: rust_decimal::Decimal::new(1, 0),
                },
            )]])
            .into_connection();

        assert_eq!(
            crate::database::task_property::Entity::find()
                .find_also_linked(crate::database::task_num_property::TaskPropertyLink)
                .all(&db_connection)
                .await
                .unwrap(),
            [(
                crate::database::task_property::Model {
                    task_id: 1,
                    name: "test".to_owned(),
                    typ: "num".to_owned(),
                },
                Some(task_num_property::Model {
                    task_id: 1,
                    name: "test".to_owned(),
                    value: rust_decimal::Decimal::new(1, 0),
                }),
            )]
        )
    }
    #[test]
    fn test_linked_to_task_property() {
        let link = TaskPropertyLink;
        let relations = link.link();
        assert_eq!(relations.len(), 4);
    }
}
