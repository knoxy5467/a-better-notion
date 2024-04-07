use actix_web::rt::task;
use sea_orm::{entity::prelude::*, RelationBuilder, RelationType};

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
}
