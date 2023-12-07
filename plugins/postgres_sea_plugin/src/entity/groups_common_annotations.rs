//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.6

use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "groups_common_annotations")]
pub struct Model {
    #[sea_orm(primary_key, auto_increment = false)]
    pub group_id: i32,
    #[sea_orm(primary_key, auto_increment = false)]
    pub common_annotation_id: i32,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(
        belongs_to = "super::common_annotations::Entity",
        from = "Column::CommonAnnotationId",
        to = "super::common_annotations::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    CommonAnnotations,
    #[sea_orm(
        belongs_to = "super::groups::Entity",
        from = "Column::GroupId",
        to = "super::groups::Column::Id",
        on_update = "NoAction",
        on_delete = "NoAction"
    )]
    Groups,
}

impl Related<super::common_annotations::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::CommonAnnotations.def()
    }
}

impl Related<super::groups::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::Groups.def()
    }
}

impl ActiveModelBehavior for ActiveModel {}
