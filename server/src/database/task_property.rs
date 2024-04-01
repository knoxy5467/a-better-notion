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
}
impl Related<super::task::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Task.def()
    }
}
impl ActiveModelBehavior for ActiveModel {}
