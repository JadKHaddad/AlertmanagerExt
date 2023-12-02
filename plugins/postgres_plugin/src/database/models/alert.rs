use super::alert_status::AlertStatusModel;
use crate::database::schema::{
    alert, alert_annotation, alert_label, assign_alert_annotation, assign_alert_label,
};
use diesel::prelude::*;

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

#[derive(Queryable, Selectable)]
#[diesel(table_name = alert)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Alert {
    pub group_key: String,
    pub status: AlertStatusModel,
    pub starts_at: chrono::NaiveDateTime,
    pub ends_at: Option<chrono::NaiveDateTime>,
    pub generator_url: String,
    pub fingerprint: String,
}

#[derive(Insertable)]
#[diesel(table_name = alert_label)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InsertableAlertLabel<'a> {
    pub name: &'a str,
    pub value: &'a str,
}

#[derive(Queryable, Selectable)]
#[diesel(table_name = alert_label)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AlertLabel {
    pub name: String,
    pub value: String,
}

#[derive(Insertable)]
#[diesel(table_name = assign_alert_label)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InsertableAssignAlertLabel {
    pub alert_id: i32,
    pub alert_label_id: i32,
}

#[derive(Identifiable, Selectable, Queryable, Associations, Debug, Clone)]
#[diesel(belongs_to(Alert))]
#[diesel(belongs_to(AlertLabel))]
#[diesel(table_name = assign_alert_label)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AssignAlertLabel {
    pub id: i32,
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

#[derive(Queryable, Selectable)]
#[diesel(table_name = alert_annotation)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AlertAnnotation {
    pub name: String,
    pub value: String,
}

#[derive(Insertable)]
#[diesel(table_name = assign_alert_annotation)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AssignAlertAnnotation {
    pub alert_id: i32,
    pub alert_annotation_id: i32,
}

#[derive(Queryable)]
pub struct DatabaseAlert {
    pub selectable_alert: Alert,
    pub selectable_alert_labels: Vec<AlertLabel>,
    pub selectable_alert_annotations: Vec<AlertAnnotation>,
}

impl From<(Alert, Vec<AlertLabel>, Vec<AlertAnnotation>)> for DatabaseAlert {
    fn from(
        (selectable_alert, selectable_alert_labels, selectable_alert_annotations): (
            Alert,
            Vec<AlertLabel>,
            Vec<AlertAnnotation>,
        ),
    ) -> Self {
        Self {
            selectable_alert,
            selectable_alert_labels,
            selectable_alert_annotations,
        }
    }
}
