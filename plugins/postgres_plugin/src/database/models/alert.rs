use super::alert_status::AlertStatusModel;
use crate::database::schema::{
    alert, alert_annotation, alert_label, assign_alert_annotation, assign_alert_label,
};
use diesel::Insertable;

#[derive(Insertable)]
#[diesel(table_name = alert)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InsertableAlert<'a> {
    pub alert_group_id: i32,
    pub group_key: &'a str,
    pub status: &'a AlertStatusModel,
    pub starts_at: chrono::NaiveDateTime,
    pub ends_at: Option<chrono::NaiveDateTime>,
    pub generator_url: &'a str,
    pub fingerprint: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = alert_label)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InsertableAlertLabel<'a> {
    pub name: &'a str,
    pub value: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = assign_alert_label)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AssignAlertLabel {
    pub alert_id: i32,
    pub alert_label_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = alert_annotation)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InsertableAlertAnnotation<'a> {
    pub name: &'a str,
    pub value: &'a str,
}

#[derive(Insertable)]
#[diesel(table_name = assign_alert_annotation)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AssignAlertAnnotation {
    pub alert_id: i32,
    pub alert_annotation_id: i32,
}
