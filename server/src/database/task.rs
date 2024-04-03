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
}
impl Related<super::task_property::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::TaskProperty.def()
    }
}
impl ActiveModelBehavior for ActiveModel {}

#[cfg(test)]
mod tests {
    use crate::database::task_property;

    use super::*;
    use sea_orm::{entity::prelude::*, entity::*, DatabaseBackend, MockDatabase, Transaction};
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
        assert_eq!(iter.next(), None);
    }

    #[tokio::test]
    async fn test_task_related() {
        let db = MockDatabase::new(DatabaseBackend::Postgres);
        let createdTime = chrono::offset::Utc::now().naive_utc();
        let db_connection = db
            .append_query_results([[(
                crate::database::task::Model {
                    id: 1,
                    title: "Task 1".to_owned(),
                    completed: false,
                    last_edited: createdTime,
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
                .find_also_related(task_property::Entity)
                .all(&db_connection)
                .await
                .unwrap(),
            [(
                crate::database::task::Model {
                    id: 1,
                    title: "Task 1".to_owned(),
                    completed: false,
                    last_edited: createdTime,
                },
                Some(crate::database::task_property::Model {
                    task_id: 1,
                    name: "gas".to_owned(),
                    typ: "bool".to_owned(),
                })
            )]
        )
    }
}
