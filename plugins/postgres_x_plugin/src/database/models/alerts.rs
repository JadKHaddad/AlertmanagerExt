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

pub struct DatabaseAlert {
    pub alert: Alert,
    pub labels: Vec<Label>,
    pub annotations: Vec<Annotation>,
}

impl From<DatabaseAlert> for StandAloneAlert {
    fn from(database_alert: DatabaseAlert) -> Self {
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
