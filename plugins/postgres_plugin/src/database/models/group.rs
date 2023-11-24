use super::alert_status::AlertStatusModel;
use crate::database::schema::{alert_group, common_annotation, common_label, group_label};
use diesel::Insertable;

#[derive(Insertable)]
#[diesel(table_name = alert_group)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InsertableAlertGroup<'a> {
    pub group_key: &'a str,
    pub receiver: &'a str,
    pub status: &'a AlertStatusModel,
    pub external_url: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = group_label)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InsertableGroupLabel<'a> {
    pub alert_group_id: i32,
    pub name: &'a str,
    pub value: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = common_label)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InsertableCommonLabel<'a> {
    pub alert_group_id: i32,
    pub name: &'a str,
    pub value: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = common_annotation)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InsertableCommonAnnotation<'a> {
    pub alert_group_id: i32,
    pub name: &'a str,
    pub value: &'a str,
}