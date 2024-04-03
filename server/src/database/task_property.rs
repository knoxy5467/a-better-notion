use sea_orm::entity::prelude::*;
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "task_property")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub task_id: i32,
    #[sea_orm(primary_key)]
    pub name: String,
    #[sea_orm(column_name = "type")]
    pub typ: String,
}
#[derive(Copy, Clone, Debug, PartialEq, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::task::Entity",
        from = "Column::TaskId",
        to = "super::task::Column::Id"
    )]
    Task,
    #[sea_orm(has_one = "super::task_bool_property::Entity")]
    TaskBoolProperty,
    #[sea_orm(has_one = "super::task_string_property::Entity")]
    TaskStringProperty,
}
impl Related<super::task::Entity> for Entity {
    fn to() -> RelationDef {
        let relation = Relation::Task.def();
        return relation;
    }
}
impl Related<super::task_bool_property::Entity> for Entity {
    fn to() -> RelationDef {
        let relation = Relation::TaskBoolProperty.def();
        return relation;
    }
}
impl Related<super::task_string_property::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TaskStringProperty.def()
    }
}
impl ActiveModelBehavior for ActiveModel {}

#[cfg(test)]
mod tests {
    use crate::database::{task_bool_property, task_property, task_string_property};

    use super::*;
    use sea_orm::{entity::*, DatabaseBackend, MockDatabase, Transaction};

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
        assert_eq!(iter.next(), Some(Relation::TaskBoolProperty));
        assert_eq!(iter.next(), Some(Relation::TaskStringProperty));
        assert_eq!(iter.next(), None);
    }
    #[tokio::test]
    async fn test_bool_property_relation() {
        let db = MockDatabase::new(DatabaseBackend::Postgres);
        let db_conn = db
            .append_query_results([[(
                task_property::Model {
                    task_id: 1,
                    name: "gas".to_owned(),
                    typ: "bool".to_owned(),
                },
                task_bool_property::Model {
                    task_id: 1,
                    name: "gas".to_owned(),
                    value: true,
                },
            )]])
            .into_connection();

        assert_eq!(
            task_property::Entity::find()
                .find_also_related(task_bool_property::Entity)
                .all(&db_conn)
                .await
                .unwrap(),
            [(
                task_property::Model {
                    task_id: 1,
                    name: "gas".to_owned(),
                    typ: "bool".to_owned(),
                },
                Some(task_bool_property::Model {
                    task_id: 1,
                    name: "gas".to_owned(),
                    value: true,
                }),
            )]
        );
    }
    #[tokio::test]
    async fn test_string_property_relation() {
        let db = MockDatabase::new(DatabaseBackend::Postgres);
        let db_conn = db
            .append_query_results([[(
                task_property::Model {
                    task_id: 1,
                    name: "gas".to_owned(),
                    typ: "string".to_owned(),
                },
                task_string_property::Model {
                    task_id: 1,
                    name: "gas".to_owned(),
                    value: "hello".to_owned(),
                },
            )]])
            .into_connection();

        assert_eq!(
            task_property::Entity::find()
                .find_also_related(task_string_property::Entity)
                .all(&db_conn)
                .await
                .unwrap(),
            [(
                task_property::Model {
                    task_id: 1,
                    name: "gas".to_owned(),
                    typ: "string".to_owned(),
                },
                Some(task_string_property::Model {
                    task_id: 1,
                    name: "gas".to_owned(),
                    value: "hello".to_owned(),
                }),
            )]
        );
    }
}
