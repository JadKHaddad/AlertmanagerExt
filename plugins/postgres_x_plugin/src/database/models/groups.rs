use sqlx::FromRow;

use super::alert_status::AlertStatusModel;

#[derive(Debug, Clone, FromRow)]
pub struct Group {
    pub id: i32,
    pub group_key: String,
    pub receiver: String,
    pub status: AlertStatusModel,
    pub external_url: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct GroupLabel {
    pub group_id: i32,
    pub label_id: i32,
}

#[derive(Debug, Clone, FromRow)]
pub struct GroupCommonLabel {
    pub group_id: i32,
    pub common_label_id: i32,
}

#[derive(Debug, Clone, FromRow)]
pub struct GroupCommonAnnotation {
    pub group_id: i32,
    pub common_annotation_id: i32,
}
