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
        belongs_to = "task::Entity",
        to = "task::column::id",
        from = "Column::task_id"
    )]
    Task,
    #[sea_orm(has_one = "task_bool_property::Entity")]
    TaskBoolProperty,
    #[sea_orm(has_one = "task_num_property::Entity")]
    TaskNumProperty,
    #[sea_orm(has_one = "task_string_property::Entity")]
    TaskStringProperty,
    #[sea_orm(has_one = "task_date_property::Entity")]
    TaskDateProperty,
}
impl ActiveModelBehavior for ActiveModel {}
