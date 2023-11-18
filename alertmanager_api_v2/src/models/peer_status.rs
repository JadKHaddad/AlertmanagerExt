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
pub struct PeerStatus {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "address")]
    pub address: String,
}

impl PeerStatus {
    pub fn new(name: String, address: String) -> PeerStatus {
        PeerStatus {
            name,
            address,
        }
    }
}

