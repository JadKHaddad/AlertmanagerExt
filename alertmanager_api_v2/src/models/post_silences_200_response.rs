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
pub struct PostSilences200Response {
    #[serde(rename = "silenceID", skip_serializing_if = "Option::is_none")]
    pub silence_id: Option<String>,
}

impl PostSilences200Response {
    pub fn new() -> PostSilences200Response {
        PostSilences200Response {
            silence_id: None,
        }
    }
}

