use models::Status;
use mongodb::bson::oid::ObjectId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertableAlertGroup {
    pub group_key: String,
    pub truncated_alerts: i32,
    pub status: Status,
    pub receiver: String,
    pub external_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertableGroupLabel {
    pub alert_group_id: ObjectId,
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertableCommonLabel {
    pub alert_group_id: ObjectId,
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InsertableCommonAnnotation {
    pub alert_group_id: ObjectId,
    pub name: String,
    pub value: String,
}
