use crate::database::schema::{
    alert_group, assign_common_annotation, assign_common_label, assign_group_label,
    common_annotation, common_label, group_label,
};
use diesel::Insertable;

#[derive(Insertable)]
#[diesel(table_name = alert_group)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct InsertableAlertGroup {
    pub group_key: String,
    pub receiver: String,
    pub status: String,
    pub external_url: String,
}

#[derive(Insertable)]
#[diesel(table_name = group_label)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct InsertableGroupLabel {
    pub name: String,
    pub value: String,
}

#[derive(Insertable)]
#[diesel(table_name = assign_group_label)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct AssignGroupLabel {
    pub alert_group_id: i32,
    pub group_label_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = common_label)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct InsertableCommonLabel {
    pub name: String,
    pub value: String,
}

#[derive(Insertable)]
#[diesel(table_name = assign_common_label)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct AssignCommonLabel {
    pub alert_group_id: i32,
    pub common_label_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = common_annotation)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct InsertableCommonAnnotation {
    pub name: String,
    pub value: String,
}

#[derive(Insertable)]
#[diesel(table_name = assign_common_annotation)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct AssignCommonAnnotation {
    pub alert_group_id: i32,
    pub common_annotation_id: i32,
}
