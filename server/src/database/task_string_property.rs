use sea_orm::entity::prelude::*;
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "task_string_property")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub task_id: i32,
    #[sea_orm(primary_key)]
    pub name: String,
    pub value: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::task_property::Entity",
        from = "Column::TaskId",
        to = "super::task_property::Column::TaskId"
    )]
    Task,
    #[sea_orm(
        belongs_to = "super::task_property::Entity",
        from = "Column::Name",
        to = "super::task_property::Column::Name"
    )]
    Name,
}
impl Related<super::task_property::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Task.def()
    }
}
impl ActiveModelBehavior for ActiveModel {}
