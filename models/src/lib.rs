use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct AlermanagerPush {
    pub version: String,
    /// key identifying the group of alerts (e.g. to deduplicate)
    pub group_key: String,
    /// how many alerts have been truncated due to "max_alerts"
    pub truncated_alerts: i32,
    pub status: Status,
    pub receiver: String,
    pub group_labels: HashMap<String, String>,
    pub common_labels: HashMap<String, String>,
    pub common_annotations: HashMap<String, String>,
    /// backlink to the Alertmanager.
    #[serde(rename = "externalURL")]
    pub external_url: String,
    pub alerts: Vec<Alert>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Status {
    Resolved,
    Firing,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Alert {
    pub status: Status,
    pub labels: HashMap<String, String>,
    pub annotations: HashMap<String, String>,
    /// rfc3339
    pub starts_at: String,
    /// rfc3339
    pub ends_at: String,
    /// identifies the entity that caused the alert
    #[serde(rename = "generatorURL")]
    pub generator_url: String,
    /// fingerprint to identify the alert
    pub fingerprint: String,
}
