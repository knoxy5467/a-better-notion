use sea_orm::entity::prelude::*;
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "task_bool_property")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub task_id: i32,
    #[sea_orm(primary_key)]
    pub name: String,
    pub value: bool,
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
impl ActiveModelBehavior for ActiveModel {}
