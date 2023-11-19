use super::alert_status::AlertStatusModel;
use crate::database::schema::{alert, alert_annotation, alert_label};
use diesel::Insertable;

#[derive(Insertable)]
#[diesel(table_name = alert)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InsertableAlert<'a> {
    alert_group_id: i32,
    status: &'a AlertStatusModel,
    starts_at: std::time::SystemTime,
    ends_at: Option<std::time::SystemTime>,
    generator_url: &'a str,
    fingerprint: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = alert_label)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InsertableAlertLabel<'a> {
    alert_id: i32,
    name: &'a str,
    value: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = alert_annotation)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InsertableAlertAnnotation<'a> {
    alert_id: i32,
    name: &'a str,
    value: &'a str,
}
