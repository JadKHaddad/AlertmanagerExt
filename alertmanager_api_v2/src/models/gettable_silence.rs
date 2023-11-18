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
pub struct GettableSilence {
    #[serde(rename = "matchers")]
    pub matchers: Vec<crate::models::Matcher>,
    #[serde(rename = "startsAt")]
    pub starts_at: String,
    #[serde(rename = "endsAt")]
    pub ends_at: String,
    #[serde(rename = "createdBy")]
    pub created_by: String,
    #[serde(rename = "comment")]
    pub comment: String,
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "status")]
    pub status: Box<crate::models::SilenceStatus>,
    #[serde(rename = "updatedAt")]
    pub updated_at: String,
}

impl GettableSilence {
    pub fn new(matchers: Vec<crate::models::Matcher>, starts_at: String, ends_at: String, created_by: String, comment: String, id: String, status: crate::models::SilenceStatus, updated_at: String) -> GettableSilence {
        GettableSilence {
            matchers,
            starts_at,
            ends_at,
            created_by,
            comment,
            id,
            status: Box::new(status),
            updated_at,
        }
    }
}

