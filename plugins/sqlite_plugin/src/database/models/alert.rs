use crate::database::schema::{alert, alert_annotation, alert_label};
use diesel::Insertable;

#[derive(Insertable)]
#[diesel(table_name = alert)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct InsertableAlert {
    pub alert_group_id: i32,
    pub status: String,
    pub starts_at: chrono::NaiveDateTime,
    pub ends_at: Option<chrono::NaiveDateTime>,
    pub generator_url: String,
    pub fingerprint: String,
}

#[derive(Insertable)]
#[diesel(table_name = alert_label)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct InsertableAlertLabel {
    pub alert_id: i32,
    pub name: String,
    pub value: String,
}

#[derive(Insertable)]
#[diesel(table_name = alert_annotation)]
#[diesel(check_for_backend(diesel::sqlite::Sqlite))]
pub struct InsertableAlertAnnotation {
    pub alert_id: i32,
    pub name: String,
    pub value: String,
}
