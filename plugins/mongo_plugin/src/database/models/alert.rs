use models::Status;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertableAlert {
    pub status: Status,
    pub starts_at: chrono::NaiveDateTime,
    pub ends_at: Option<chrono::NaiveDateTime>,
    pub generator_url: String,
    pub fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertableAlertLabel {
    pub alert_id: ObjectId,
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertableAlertAnnotation {
    pub alert_id: ObjectId,
    pub name: String,
    pub value: String,
}
