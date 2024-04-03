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
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
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
