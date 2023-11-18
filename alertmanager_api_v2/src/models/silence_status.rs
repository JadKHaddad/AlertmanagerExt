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
pub struct SilenceStatus {
    #[serde(rename = "state")]
    pub state: State,
}

impl SilenceStatus {
    pub fn new(state: State) -> SilenceStatus {
        SilenceStatus {
            state,
        }
    }
}

/// 
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
pub enum State {
    #[serde(rename = "expired")]
    Expired,
    #[serde(rename = "active")]
    Active,
    #[serde(rename = "pending")]
    Pending,
}

impl Default for State {
    fn default() -> State {
        Self::Expired
    }
}
