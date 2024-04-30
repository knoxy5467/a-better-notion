use common::Filter;
use sea_orm::entity::prelude::*;
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "view")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub properties: Vec<String>,
    pub filter: Filter,
}
impl ActiveModelBehavior for ActiveModel {}
#[cfg(test)]
mod view_tests {
    use super::*;
    use sea_orm::Iterable;
    #[test]
    fn test_copy_clone_debug_derives() {
        let original = Model {
            id: 1,
            properties: vec!["name".to_string()],
            filter: Filter::Equal("name".to_string(), "John".to_string()),
        };
        let copy = original;
        assert_eq!(original, copy);
        let clone = original.clone();
        assert_eq!(original, clone);
        format!("{:?}", original);
    }
}
