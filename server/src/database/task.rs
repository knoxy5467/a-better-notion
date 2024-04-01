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
#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
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
