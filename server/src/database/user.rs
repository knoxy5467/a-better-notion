use sea_orm::entity::prelude::*;
#[derive(Clone, Debug, PartialEq, DeriveEntityModel)]
#[sea_orm(table_name = "user")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub email: String,
    // this is the hashed password, never store plaintext
    pub password_hash: String,
}
impl ActiveModelBehavior for ActiveModel {}

#[cfg(test)]
mod user_tests {
    use super::*;
    use sea_orm::entity::prelude::*;
    #[test]
    fn test_copy_clone_debug_derives() {
        let original = Model {
            id: 1,
            email: "".to_owned(),
            password_hash: "".to_owned(),
        };
        let copy = original;
        assert_eq!(original, copy);
        let clone = original.clone();
        assert_eq!(original, clone);
        format!("{:?}", original);
    }
}
