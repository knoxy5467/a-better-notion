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
    #[sea_orm(has_many = "super::task::Entity")]
    DependsOn,
}
impl Related<super::task::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Task.def()
    }
    fn via() -> Option<RelationDef> {
        Some(Relation::DependsOn.def())
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
