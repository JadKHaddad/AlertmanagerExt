use super::alert_status::AlertStatusModel;
use crate::database::models::{annotations::Annotation, labels::Label};
use crate::database::schema::{alerts, alerts_annotations, alerts_labels};
use diesel::prelude::*;
use models::{Alert as AlertmanagerPushAlert, StandAloneAlert};
use sqlx::FromRow;

#[derive(Insertable, Debug)]
#[diesel(table_name = alerts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InsertableAlert<'a> {
    pub group_id: i32,
    pub group_key: &'a str,
    pub status: &'a AlertStatusModel,
    pub starts_at: chrono::NaiveDateTime,
    pub ends_at: Option<chrono::NaiveDateTime>,
    pub generator_url: &'a str,
    pub fingerprint: &'a str,
}

#[derive(Queryable, Selectable, Identifiable, PartialEq, Debug, Clone, FromRow)]
#[diesel(table_name = alerts)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct Alert {
    pub id: i32,
    pub group_key: String,
    pub status: AlertStatusModel,
    pub starts_at: chrono::NaiveDateTime,
    pub ends_at: Option<chrono::NaiveDateTime>,
    pub generator_url: String,
    pub fingerprint: String,
}

#[derive(Insertable)]
#[diesel(table_name = alerts_labels)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InsertableAlertLabel {
    pub alert_id: i32,
    pub label_id: i32,
}

#[derive(Identifiable, Selectable, Queryable, Associations, Debug, Clone, FromRow)]
#[diesel(belongs_to(Alert))]
#[diesel(belongs_to(Label))]
#[diesel(table_name = alerts_labels)]
#[diesel(primary_key(alert_id, label_id))]
pub struct AlertLabel {
    pub alert_id: i32,
    pub label_id: i32,
}

#[derive(Insertable)]
#[diesel(table_name = alerts_annotations)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct InsertableAlertAnnotation {
    pub alert_id: i32,
    pub annotation_id: i32,
}

#[derive(Identifiable, Selectable, Queryable, Associations, Debug, Clone, FromRow)]
#[diesel(belongs_to(Alert))]
#[diesel(belongs_to(Annotation))]
#[diesel(table_name = alerts_annotations)]
#[diesel(primary_key(alert_id, annotation_id))]
pub struct AlertAnnotation {
    pub alert_id: i32,
    pub annotation_id: i32,
}

#[derive(Queryable)]
pub struct DieselDatabaseAlert {
    pub alert: Alert,
    pub labels: Vec<Label>,
    pub annotations: Vec<Annotation>,
}

impl From<DieselDatabaseAlert> for StandAloneAlert {
    fn from(database_alert: DieselDatabaseAlert) -> Self {
        StandAloneAlert {
            group_key: database_alert.alert.group_key,
            alert: AlertmanagerPushAlert {
                status: database_alert.alert.status.into(),
                labels: database_alert
                    .labels
                    .into_iter()
                    .map(|label| (label.name, label.value))
                    .collect(),
                annotations: database_alert
                    .annotations
                    .into_iter()
                    .map(|annotation| (annotation.name, annotation.value))
                    .collect(),
                starts_at: chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
                    database_alert.alert.starts_at,
                    chrono::Utc,
                )
                .to_rfc3339(),
                ends_at: database_alert
                    .alert
                    .ends_at
                    .map(|ends_at| {
                        chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
                            ends_at,
                            chrono::Utc,
                        )
                        .to_rfc3339()
                    })
                    .unwrap_or_default(),
                generator_url: database_alert.alert.generator_url,
                fingerprint: database_alert.alert.fingerprint,
            },
        }
    }
}

#[derive(Debug, Clone, FromRow)]
/// Had to flatten the Alert struct because [`sqlx::query_as!`] doesn't support `#[sqlx(flatten)]`
/// We also had to use Options fot the vectors
pub struct SqlxDatabaseAlert {
    pub id: i32,
    pub group_key: String,
    pub status: AlertStatusModel,
    pub starts_at: chrono::NaiveDateTime,
    pub ends_at: Option<chrono::NaiveDateTime>,
    pub generator_url: String,
    pub fingerprint: String,
    pub labels: Option<Vec<Label>>,
    pub annotations: Option<Vec<Annotation>>,
}

impl From<SqlxDatabaseAlert> for StandAloneAlert {
    fn from(database_alert: SqlxDatabaseAlert) -> Self {
        StandAloneAlert {
            group_key: database_alert.group_key,
            alert: AlertmanagerPushAlert {
                status: database_alert.status.into(),
                labels: database_alert
                    .labels
                    .unwrap_or(Vec::new())
                    .into_iter()
                    .map(|label| (label.name, label.value))
                    .collect(),
                annotations: database_alert
                    .annotations
                    .unwrap_or(Vec::new())
                    .into_iter()
                    .map(|annotation| (annotation.name, annotation.value))
                    .collect(),
                starts_at: chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
                    database_alert.starts_at,
                    chrono::Utc,
                )
                .to_rfc3339(),
                ends_at: database_alert
                    .ends_at
                    .map(|ends_at| {
                        chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
                            ends_at,
                            chrono::Utc,
                        )
                        .to_rfc3339()
                    })
                    .unwrap_or_default(),
                generator_url: database_alert.generator_url,
                fingerprint: database_alert.fingerprint,
            },
        }
    }
}
