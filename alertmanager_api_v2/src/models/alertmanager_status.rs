/*
 * Alertmanager API
 *
 * API of the Prometheus Alertmanager (https://github.com/prometheus/alertmanager)
 *
 * The version of the OpenAPI document: 0.0.1
 * 
 * Generated by: https://openapi-generator.tech
 */




#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AlertmanagerStatus {
    #[serde(rename = "cluster")]
    pub cluster: Box<crate::models::ClusterStatus>,
    #[serde(rename = "versionInfo")]
    pub version_info: Box<crate::models::VersionInfo>,
    #[serde(rename = "config")]
    pub config: Box<crate::models::AlertmanagerConfig>,
    #[serde(rename = "uptime")]
    pub uptime: String,
}

impl AlertmanagerStatus {
    pub fn new(cluster: crate::models::ClusterStatus, version_info: crate::models::VersionInfo, config: crate::models::AlertmanagerConfig, uptime: String) -> AlertmanagerStatus {
        AlertmanagerStatus {
            cluster: Box::new(cluster),
            version_info: Box::new(version_info),
            config: Box::new(config),
            uptime,
        }
    }
}


