use super::alert_status::AlertStatusModel;
use crate::database::schema::{alert_group, common_annotation, common_label, group_label};
use diesel::Insertable;

#[derive(Insertable)]
#[diesel(table_name = alert_group)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InsertableAlertGroup<'a> {
    group_key: &'a str,
    receiver: &'a str,
    status: &'a AlertStatusModel,
    external_url: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = group_label)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InsertableGroupLabel<'a> {
    alert_group_id: i32,
    name: &'a str,
    value: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = common_label)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InsertableCommonLabel<'a> {
    alert_group_id: i32,
    name: &'a str,
    value: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = common_annotation)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InsertableCommonAnnotation<'a> {
    alert_group_id: i32,
    name: &'a str,
    value: &'a str,
}
