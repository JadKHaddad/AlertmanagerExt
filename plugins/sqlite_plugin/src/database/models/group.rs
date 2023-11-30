use crate::database::schema::{alert_group, common_annotation, common_label, group_label};
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
    pub alert_group_id: i32,
    pub name: String,
    pub value: String,
}

#[derive(Insertable)]
#[diesel(table_name = common_label)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct InsertableCommonLabel {
    pub alert_group_id: i32,
    pub name: String,
    pub value: String,
}

#[derive(Insertable)]
#[diesel(table_name = common_annotation)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct InsertableCommonAnnotation {
    pub alert_group_id: i32,
    pub name: String,
    pub value: String,
}
