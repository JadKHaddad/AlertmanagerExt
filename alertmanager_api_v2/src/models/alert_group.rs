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
pub struct AlertGroup {
    #[serde(rename = "labels")]
    pub labels: ::std::collections::HashMap<String, String>,
    #[serde(rename = "receiver")]
    pub receiver: Box<crate::models::Receiver>,
    #[serde(rename = "alerts")]
    pub alerts: Vec<crate::models::GettableAlert>,
}

impl AlertGroup {
    pub fn new(labels: ::std::collections::HashMap<String, String>, receiver: crate::models::Receiver, alerts: Vec<crate::models::GettableAlert>) -> AlertGroup {
        AlertGroup {
            labels,
            receiver: Box::new(receiver),
            alerts,
        }
    }
}


