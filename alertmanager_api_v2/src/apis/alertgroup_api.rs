/*
 * Alertmanager API
 *
 * API of the Prometheus Alertmanager (https://github.com/prometheus/alertmanager)
 *
 * The version of the OpenAPI document: 0.0.1
 * 
 * Generated by: https://openapi-generator.tech
 */


use reqwest;

use crate::apis::ResponseContent;
use super::{Error, configuration};


/// struct for typed errors of method [`get_alert_groups`]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum GetAlertGroupsError {
    Status400(String),
    Status500(String),
    UnknownValue(serde_json::Value),
}


/// Get a list of alert groups
pub async fn get_alert_groups(configuration: &configuration::Configuration, active: Option<bool>, silenced: Option<bool>, inhibited: Option<bool>, filter: Option<Vec<String>>, receiver: Option<&str>) -> Result<Vec<crate::models::AlertGroup>, Error<GetAlertGroupsError>> {
    let local_var_configuration = configuration;

    let local_var_client = &local_var_configuration.client;

    let local_var_uri_str = format!("{}/alerts/groups", local_var_configuration.base_path);
    let mut local_var_req_builder = local_var_client.request(reqwest::Method::GET, local_var_uri_str.as_str());

    if let Some(ref local_var_str) = active {
        local_var_req_builder = local_var_req_builder.query(&[("active", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = silenced {
        local_var_req_builder = local_var_req_builder.query(&[("silenced", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = inhibited {
        local_var_req_builder = local_var_req_builder.query(&[("inhibited", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_str) = filter {
        local_var_req_builder = match "multi" {
            "multi" => local_var_req_builder.query(&local_var_str.into_iter().map(|p| ("filter".to_owned(), p.to_string())).collect::<Vec<(std::string::String, std::string::String)>>()),
            _ => local_var_req_builder.query(&[("filter", &local_var_str.into_iter().map(|p| p.to_string()).collect::<Vec<String>>().join(",").to_string())]),
        };
    }
    if let Some(ref local_var_str) = receiver {
        local_var_req_builder = local_var_req_builder.query(&[("receiver", &local_var_str.to_string())]);
    }
    if let Some(ref local_var_user_agent) = local_var_configuration.user_agent {
        local_var_req_builder = local_var_req_builder.header(reqwest::header::USER_AGENT, local_var_user_agent.clone());
    }

    let local_var_req = local_var_req_builder.build()?;
    let local_var_resp = local_var_client.execute(local_var_req).await?;

    let local_var_status = local_var_resp.status();
    let local_var_content = local_var_resp.text().await?;

    if !local_var_status.is_client_error() && !local_var_status.is_server_error() {
        serde_json::from_str(&local_var_content).map_err(Error::from)
    } else {
        let local_var_entity: Option<GetAlertGroupsError> = serde_json::from_str(&local_var_content).ok();
        let local_var_error = ResponseContent { status: local_var_status, content: local_var_content, entity: local_var_entity };
        Err(Error::ResponseError(local_var_error))
    }
}
