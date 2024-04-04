use sea_orm::entity::prelude::*;
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "task")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub title: String,
    pub completed: bool,
    pub last_edited: chrono::NaiveDateTime,
}
#[derive(Copy, Clone, Debug, EnumIter, PartialEq, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::task_property::Entity")]
    TaskProperty,
    #[sea_orm(has_many = "super::task_string_property::Entity")]
    TaskStringProperty,
    #[sea_orm(has_many = "super::task_bool_property::Entity")]
    TaskBoolProperty,
    #[sea_orm(has_many = "super::task_num_property::Entity")]
    TaskNumProperty,
    #[sea_orm(has_many = "super::task_date_property::Entity")]
    TaskDateProperty,
    #[sea_orm(has_many = "super::dependency::Entity")]
    Dependency,
}
impl Related<super::task_property::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TaskProperty.def()
    }
}
impl Related<super::task_string_property::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TaskStringProperty.def()
    }
}
impl Related<super::task_bool_property::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TaskBoolProperty.def()
    }
}
impl Related<super::task_num_property::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TaskNumProperty.def()
    }
}
impl Related<super::task_date_property::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TaskDateProperty.def()
    }
}
impl Related<super::dependency::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Dependency.def()
    }
}
impl ActiveModelBehavior for ActiveModel {}

#[cfg(test)]
mod tests {
    use super::*;
    use sea_orm::{DatabaseBackend, Iterable, MockDatabase};
    #[test]
    fn test_copy_clone_debug() {
        let original = Relation::TaskProperty;
        let copy = original;
        assert_eq!(original, copy);
        let clone = original.clone();
        assert_eq!(original, clone);
        format!("{:?}", original);
    }
    #[test]
    fn test_enum_iter() {
        let mut iter = Relation::iter();
        assert_eq!(iter.next(), Some(Relation::TaskProperty));
        assert_eq!(iter.next(), Some(Relation::TaskStringProperty));
        assert_eq!(iter.next(), Some(Relation::TaskBoolProperty));
        assert_eq!(iter.next(), Some(Relation::TaskNumProperty));
        assert_eq!(iter.next(), Some(Relation::TaskDateProperty));
        assert_eq!(iter.next(), Some(Relation::Dependency));
        assert_eq!(iter.next(), None);
    }

    #[tokio::test]
    async fn test_task_related() {
        let db = MockDatabase::new(DatabaseBackend::Postgres);
        let created_time = chrono::offset::Utc::now().naive_utc();
        let db_connection = db
            .append_query_results([[(
                crate::database::task::Model {
                    id: 1,
                    title: "Task 1".to_owned(),
                    completed: false,
                    last_edited: created_time,
                },
                crate::database::task_property::Model {
                    task_id: 1,
                    name: "gas".to_owned(),
                    typ: "bool".to_owned(),
                },
            )]])
            .into_connection();

        assert_eq!(
            crate::database::task::Entity::find()
                .find_also_related(crate::database::task_property::Entity)
                .all(&db_connection)
                .await
                .unwrap(),
            [(
                crate::database::task::Model {
                    id: 1,
                    title: "Task 1".to_owned(),
                    completed: false,
                    last_edited: created_time,
                },
                Some(crate::database::task_property::Model {
                    task_id: 1,
                    name: "gas".to_owned(),
                    typ: "bool".to_owned(),
                })
            )]
        )
    }
    #[tokio::test]
    async fn test_task_string_property_related() {
        let db = MockDatabase::new(DatabaseBackend::Postgres);
        let created_time = chrono::offset::Utc::now().naive_utc();
        let db_connection = db
            .append_query_results([[(
                crate::database::task::Model {
                    id: 1,
                    title: "Task 1".to_owned(),
                    completed: false,
                    last_edited: created_time,
                },
                crate::database::task_string_property::Model {
                    task_id: 1,
                    name: "gas".to_owned(),
                    value: "bool".to_owned(),
                },
            )]])
            .into_connection();

        assert_eq!(
            crate::database::task::Entity::find()
                .find_also_related(crate::database::task_string_property::Entity)
                .all(&db_connection)
                .await
                .unwrap(),
            [(
                crate::database::task::Model {
                    id: 1,
                    title: "Task 1".to_owned(),
                    completed: false,
                    last_edited: created_time,
                },
                Some(crate::database::task_string_property::Model {
                    task_id: 1,
                    name: "gas".to_owned(),
                    value: "bool".to_owned(),
                })
            )]
        )
    }
    #[tokio::test]
    async fn test_task_bool_property_related() {
        let db = MockDatabase::new(DatabaseBackend::Postgres);
        let created_time = chrono::offset::Utc::now().naive_utc();
        let db_connection = db
            .append_query_results([[(
                crate::database::task::Model {
                    id: 1,
                    title: "Task 1".to_owned(),
                    completed: false,
                    last_edited: created_time,
                },
                crate::database::task_bool_property::Model {
                    task_id: 1,
                    name: "gas".to_owned(),
                    value: true,
                },
            )]])
            .into_connection();

        assert_eq!(
            crate::database::task::Entity::find()
                .find_also_related(crate::database::task_bool_property::Entity)
                .all(&db_connection)
                .await
                .unwrap(),
            [(
                crate::database::task::Model {
                    id: 1,
                    title: "Task 1".to_owned(),
                    completed: false,
                    last_edited: created_time,
                },
                Some(crate::database::task_bool_property::Model {
                    task_id: 1,
                    name: "gas".to_owned(),
                    value: true,
                })
            )]
        )
    }
    #[tokio::test]
    async fn test_task_num_property_related() {
        let db = MockDatabase::new(DatabaseBackend::Postgres);
        let created_time = chrono::offset::Utc::now().naive_utc();
        let db_connection = db
            .append_query_results([[(
                crate::database::task::Model {
                    id: 1,
                    title: "Task 1".to_owned(),
                    completed: false,
                    last_edited: created_time,
                },
                crate::database::task_num_property::Model {
                    task_id: 1,
                    name: "gas".to_owned(),
                    value: rust_decimal::Decimal::from_str_exact("100.001").unwrap(),
                },
            )]])
            .into_connection();

        assert_eq!(
            crate::database::task::Entity::find()
                .find_also_related(crate::database::task_num_property::Entity)
                .all(&db_connection)
                .await
                .unwrap(),
            [(
                crate::database::task::Model {
                    id: 1,
                    title: "Task 1".to_owned(),
                    completed: false,
                    last_edited: created_time,
                },
                Some(crate::database::task_num_property::Model {
                    task_id: 1,
                    name: "gas".to_owned(),
                    value: rust_decimal::Decimal::from_str_exact("100.001").unwrap(),
                })
            )]
        )
    }
    #[tokio::test]
    async fn test_task_date_property_related() {
        let db = MockDatabase::new(DatabaseBackend::Postgres);
        let created_time = chrono::offset::Utc::now().naive_utc();
        let db_connection = db
            .append_query_results([[(
                crate::database::task::Model {
                    id: 1,
                    title: "Task 1".to_owned(),
                    completed: false,
                    last_edited: created_time,
                },
                crate::database::task_date_property::Model {
                    task_id: 1,
                    name: "gas".to_owned(),
                    value: created_time,
                },
            )]])
            .into_connection();

        assert_eq!(
            crate::database::task::Entity::find()
                .find_also_related(crate::database::task_date_property::Entity)
                .all(&db_connection)
                .await
                .unwrap(),
            [(
                crate::database::task::Model {
                    id: 1,
                    title: "Task 1".to_owned(),
                    completed: false,
                    last_edited: created_time,
                },
                Some(crate::database::task_date_property::Model {
                    task_id: 1,
                    name: "gas".to_owned(),
                    value: created_time,
                })
            )]
        )
    }
    #[tokio::test]
    async fn test_task_dependency_related() {
        let db = MockDatabase::new(DatabaseBackend::Postgres);
        let created_time = chrono::offset::Utc::now().naive_utc();
        let db_connection = db
            .append_query_results([[(
                crate::database::task::Model {
                    id: 1,
                    title: "Task 1".to_owned(),
                    completed: false,
                    last_edited: created_time,
                },
                crate::database::dependency::Model {
                    task_id: 1,
                    depends_on_id: 2,
                },
            )]])
            .into_connection();

        assert_eq!(
            crate::database::task::Entity::find()
                .find_also_related(crate::database::dependency::Entity)
                .all(&db_connection)
                .await
                .unwrap(),
            [(
                crate::database::task::Model {
                    id: 1,
                    title: "Task 1".to_owned(),
                    completed: false,
                    last_edited: created_time,
                },
                Some(crate::database::dependency::Model {
                    task_id: 1,
                    depends_on_id: 2,
                })
            )]
        )
    }
}
