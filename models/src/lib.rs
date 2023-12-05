use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use utoipa::ToSchema;

pub mod utils;

#[derive(JsonSchema, Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
/// Alertmanager webhook payload
///
/// https://prometheus.io/docs/alerting/latest/configuration/#webhook_config
pub struct AlertmanagerPush {
    pub version: String,
    /// key identifying the group of alerts (e.g. to deduplicate)
    pub group_key: String,
    /// how many alerts have been truncated due to "max_alerts"
    pub truncated_alerts: i32,
    pub status: Status,
    pub receiver: String,
    pub group_labels: BTreeMap<String, String>,
    pub common_labels: BTreeMap<String, String>,
    pub common_annotations: BTreeMap<String, String>,
    /// backlink to the Alertmanager.
    #[serde(rename = "externalURL")]
    pub external_url: String,
    pub alerts: Vec<Alert>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, ToSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub enum Status {
    Resolved,
    Firing,
}

#[derive(JsonSchema)]
#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
pub struct Alert {
    pub status: Status,
    pub labels: BTreeMap<String, String>,
    pub annotations: BTreeMap<String, String>,
    /// rfc3339
    #[serde_as(as = "serde_with::chrono::DateTime<serde_with::chrono::Utc>")]
    pub starts_at: chrono::NaiveDateTime,
    /// rfc3339
    #[serde_as(as = "Option<serde_with::chrono::DateTime<serde_with::chrono::Utc>>")]
    pub ends_at: Option<chrono::NaiveDateTime>,
    /// identifies the entity that caused the alert
    #[serde(rename = "generatorURL")]
    pub generator_url: String,
    /// fingerprint to identify the alert
    pub fingerprint: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, ToSchema, PartialEq, Eq, Hash)]
#[serde(rename_all = "camelCase")]
/// An alert that can be sent to/retrieved from a plugin
pub struct StandAloneAlert {
    /// key identifying the group of alerts (e.g. to deduplicate)
    pub group_key: String,
    pub alert: Alert,
}
