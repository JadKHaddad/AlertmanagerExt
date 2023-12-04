use super::alert_status::AlertStatusModel;
use crate::database::models::{
    annotations::CommonAnnotation,
    labels::{CommonLabel, Label},
};
use crate::database::schema::{
    groups, groups_common_annotations, groups_common_labels, groups_labels,
};
use diesel::prelude::*;
use sqlx::FromRow;

#[derive(Insertable)]
#[diesel(table_name = groups)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InsertableGroup<'a> {
    pub group_key: &'a str,
    pub receiver: &'a str,
    pub status: &'a AlertStatusModel,
    pub external_url: &'a str,
}

#[derive(Queryable, Selectable, Identifiable, PartialEq, Debug, Clone, FromRow)]
#[diesel(table_name = groups)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Group {
    pub id: i32,
    pub group_key: String,
    pub receiver: String,
    pub status: AlertStatusModel,
    pub external_url: String,
}

#[derive(Insertable)]
#[diesel(table_name = groups_labels)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InsertableGroupLabel {
    pub group_id: i32,
    pub label_id: i32,
}

#[derive(Identifiable, Selectable, Queryable, Associations, Debug, Clone, FromRow)]
#[diesel(belongs_to(Group))]
#[diesel(belongs_to(Label))]
#[diesel(table_name = groups_labels)]
#[diesel(primary_key(group_id, label_id))]
pub struct GroupLabel {
    pub group_id: i32,
    pub label_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = groups_common_labels)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InsertableGroupCommonLabel {
    pub group_id: i32,
    pub common_label_id: i32,
}

#[derive(Identifiable, Selectable, Queryable, Associations, Debug, Clone, FromRow)]
#[diesel(belongs_to(Group))]
#[diesel(belongs_to(CommonLabel))]
#[diesel(table_name = groups_common_labels)]
#[diesel(primary_key(group_id, common_label_id))]
pub struct GroupCommonLabel {
    pub group_id: i32,
    pub common_label_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = groups_common_annotations)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InsertableGroupCommonAnnotation {
    pub group_id: i32,
    pub common_annotation_id: i32,
}

#[derive(Identifiable, Selectable, Queryable, Associations, Debug, Clone, FromRow)]
#[diesel(belongs_to(Group))]
#[diesel(belongs_to(CommonAnnotation))]
#[diesel(table_name = groups_common_annotations)]
#[diesel(primary_key(group_id, common_annotation_id))]
pub struct GroupCommonAnnotation {
    pub group_id: i32,
    pub common_annotation_id: i32,
}
