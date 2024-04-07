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
}
impl Related<super::task::Entity> for Entity {
    fn to() -> RelationDef {
        let relation = Relation::Task.def();
    }
}
impl ActiveModelBehavior for ActiveModel {}

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
        assert_eq!(iter.next(), None);
    }
}
