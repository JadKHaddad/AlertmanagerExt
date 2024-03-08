//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.6

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "common_annotations")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub name: String,
    pub value: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::groups_common_annotations::Entity")]
    GroupsCommonAnnotations,
}

impl Related<super::groups_common_annotations::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::GroupsCommonAnnotations.def()
    }
}

impl Related<super::groups::Entity> for Entity {
    fn to() -> RelationDef {
        super::groups_common_annotations::Relation::Groups.def()
    }
    fn via() -> Option<RelationDef> {
        Some(
            super::groups_common_annotations::Relation::CommonAnnotations
                .def()
                .rev(),
        )
    }
}

impl ActiveModelBehavior for ActiveModel {}
