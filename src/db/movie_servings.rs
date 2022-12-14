//! SeaORM Entity. Generated by sea-orm-codegen 0.10.1

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "movie_servings")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub path: String,
    pub last_timestamp: i64,
    pub last_file_position: i64,
    pub file_length: i64,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {}

impl ActiveModelBehavior for ActiveModel {}
