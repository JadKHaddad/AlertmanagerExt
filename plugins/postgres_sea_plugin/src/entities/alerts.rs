//! `SeaORM` Entity. Generated by sea-orm-codegen 0.12.6

use super::sea_orm_active_enums::AlertStatus;
use sea_orm::entity::prelude::*;

#[derive(Clone, Debug, PartialEq, DeriveEntityModel, Eq)]
#[sea_orm(table_name = "alerts")]
pub struct Model {
    #[sea_orm(primary_key)]
    pub id: i32,
    pub group_id: i32,
    pub group_key: String,
    pub status: AlertStatus,
    pub starts_at: DateTime,
    pub ends_at: Option<DateTime>,
    pub generator_url: String,
    pub fingerprint: String,
}

#[derive(Copy, Clone, Debug, EnumIter, DeriveRelation)]
pub enum Relation {
    #[sea_orm(has_many = "super::alerts_annotations::Entity")]
    AlertsAnnotations,
    #[sea_orm(has_many = "super::alerts_labels::Entity")]
    AlertsLabels,
    #[sea_orm(
        belongs_to = "super::groups::Entity",
        from = "Column::GroupId",
        to = "super::groups::Column::Id",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Groups2,
    #[sea_orm(
        belongs_to = "super::groups::Entity",
        from = "Column::GroupKey",
        to = "super::groups::Column::GroupKey",
        on_update = "Cascade",
        on_delete = "Cascade"
    )]
    Groups1,
}

impl Related<super::alerts_annotations::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AlertsAnnotations.def()
    }
}

impl Related<super::alerts_labels::Entity> for Entity {
    fn to() -> RelationDef {
        Relation::AlertsLabels.def()
    }
}

impl Related<super::annotations::Entity> for Entity {
    fn to() -> RelationDef {
        super::alerts_annotations::Relation::Annotations.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::alerts_annotations::Relation::Alerts.def().rev())
    }
}

impl Related<super::labels::Entity> for Entity {
    fn to() -> RelationDef {
        super::alerts_labels::Relation::Labels.def()
    }
    fn via() -> Option<RelationDef> {
        Some(super::alerts_labels::Relation::Alerts.def().rev())
    }
}

impl ActiveModelBehavior for ActiveModel {}
