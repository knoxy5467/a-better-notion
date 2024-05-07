use sea_orm::entity::prelude::*;
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "view")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub properties: Vec<String>,
    pub filter: String,
}

#[derive(Copy, Clone, Debug, EnumIter, PartialEq, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
#[cfg(test)]
mod view_tests {
    use super::*;
    #[test]
    fn test_copy_clone_debug_derives() {
        let original = Model {
            id: 1,
            name: "nothing".to_owned(),
            properties: vec!["name".to_string()],
            filter: "whatever".to_owned(),
        };
        let copy = original.clone();
        assert_eq!(original, copy);
        let clone = original.clone();
        assert_eq!(original, clone);
        format!("{:?}", original);
    }
}
