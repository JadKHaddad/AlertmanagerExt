use super::{alert_status::AlertStatusModel, annotations::Annotation, labels::Label};
use models::{Alert as AlertmanagerPushAlert, StandAloneAlert};
use sqlx::FromRow;

#[derive(Debug, Clone, FromRow)]
pub struct Alert {
    pub id: i32,
    pub group_key: String,
    pub status: AlertStatusModel,
    pub starts_at: chrono::NaiveDateTime,
    pub ends_at: Option<chrono::NaiveDateTime>,
    pub generator_url: String,
    pub fingerprint: String,
}

#[derive(Debug, Clone, FromRow)]
pub struct AlertLabel {
    pub alert_id: i32,
    pub label_id: i32,
}

#[derive(Debug, Clone, FromRow)]
pub struct AlertAnnotation {
    pub alert_id: i32,
    pub annotation_id: i32,
}

#[derive(Debug, Clone, FromRow)]
/// Had to flatten the Alert struct because [`sqlx::query_as!`] doesn't support `#[sqlx(flatten)]`
/// /// We also had to use Options fot the vectors
pub struct DatabaseAlert {
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

impl From<DatabaseAlert> for StandAloneAlert {
    fn from(database_alert: DatabaseAlert) -> Self {
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
                starts_at: database_alert.starts_at,
                ends_at: database_alert.ends_at,
                generator_url: database_alert.generator_url,
                fingerprint: database_alert.fingerprint,
            },
        }
    }
}
