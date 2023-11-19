use aide::{transform::TransformOperation, OperationIo};
use axum::{http::StatusCode, response::IntoResponse};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    extractors::ApiJson,
    traits::{HasOperationDocs, HasStatusCode},
};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, OperationIo, PartialEq)]
#[serde(rename_all = "camelCase")]
/// Push status
pub enum PushStatus {
    /// Push was successful
    Ok,
    /// Some alerts were pushed successfully
    Partial,
    /// Push failed
    Failed,
}

impl HasStatusCode for PushStatus {
    fn status_code(&self) -> StatusCode {
        match self {
            PushStatus::Ok => StatusCode::ACCEPTED,
            PushStatus::Partial => StatusCode::MULTI_STATUS,
            PushStatus::Failed => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, OperationIo, PartialEq)]
#[serde(rename_all = "camelCase")]
#[serde(tag = "type", content = "error")]
/// Push status for a plugin
pub enum PluginPushStatus {
    /// Push was successful
    Ok,
    /// Plugin was not found
    NotFound,
    /// Push failed
    Failed {
        /// Error message
        error_message: String,
    },
}

impl HasStatusCode for PluginPushStatus {
    fn status_code(&self) -> StatusCode {
        match self {
            PluginPushStatus::Ok => StatusCode::ACCEPTED,
            PluginPushStatus::NotFound => StatusCode::NOT_FOUND,
            PluginPushStatus::Failed { .. } => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, OperationIo)]
#[serde(rename_all = "camelCase")]
/// Response for a plugin push
pub struct PluginPushResponse {
    /// Status of the push for the plugin
    pub status: PluginPushStatus,
    /// Name of the plugin
    pub plugin_name: String,
}

impl HasOperationDocs for PluginPushResponse {
    fn operation_docs(op: TransformOperation) -> TransformOperation {
        op.description("Push alerts to a plugin")
            .response_with::<201, ApiJson<Self>, _>(|res| {
                res.description("Alerts were pushed successfully").example({
                    PluginPushResponse {
                        status: PluginPushStatus::Ok,
                        plugin_name: "Plugin 1".to_string(),
                    }
                })
            })
            .response_with::<500, ApiJson<Self>, _>(|res| {
                res.description("Failed to push alerts")
                    .example(PluginPushResponse {
                        status: PluginPushStatus::Failed {
                            error_message: "Some error".to_string(),
                        },
                        plugin_name: "Plugin 1".to_string(),
                    })
            })
    }
}

impl IntoResponse for PluginPushResponse {
    fn into_response(self) -> axum::response::Response {
        (self.status.status_code(), ApiJson(self)).into_response()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, OperationIo)]
#[serde(rename_all = "camelCase")]
/// Response for a push
pub struct PushResponse {
    /// Status of the push
    pub status: PushStatus,
    /// Responses for each plugin
    pub plugins: Vec<PluginPushResponse>,
}

impl HasOperationDocs for PushResponse {
    fn operation_docs(op: TransformOperation) -> TransformOperation {
        op.description("Push alerts to plugins")
            .response_with::<201, ApiJson<Self>, _>(|res| {
                res.description("All alerts were pushed successfully")
                    .example({
                        PushResponse {
                            status: PushStatus::Ok,
                            plugins: vec![
                                PluginPushResponse {
                                    status: PluginPushStatus::Ok,
                                    plugin_name: "Plugin 1".to_string(),
                                },
                                PluginPushResponse {
                                    status: PluginPushStatus::Ok,
                                    plugin_name: "Plugin 2".to_string(),
                                },
                            ],
                        }
                    })
            })
            .response_with::<207, ApiJson<Self>, _>(|res| {
                res.description("Some alerts were pushed successfully")
                    .example({
                        PushResponse {
                            status: PushStatus::Partial,
                            plugins: vec![
                                PluginPushResponse {
                                    status: PluginPushStatus::Ok,
                                    plugin_name: "Plugin 1".to_string(),
                                },
                                PluginPushResponse {
                                    status: PluginPushStatus::Failed {
                                        error_message: "Some error".to_string(),
                                    },
                                    plugin_name: "Plugin 2".to_string(),
                                },
                            ],
                        }
                    })
            })
            .response_with::<500, ApiJson<Self>, _>(|res| {
                res.description("Failed to push alerts")
                    .example(PushResponse {
                        status: PushStatus::Failed,
                        plugins: vec![
                            PluginPushResponse {
                                status: PluginPushStatus::Failed {
                                    error_message: "Some error".to_string(),
                                },
                                plugin_name: "Plugin 1".to_string(),
                            },
                            PluginPushResponse {
                                status: PluginPushStatus::Failed {
                                    error_message: "Some error".to_string(),
                                },
                                plugin_name: "Plugin 2".to_string(),
                            },
                        ],
                    })
            })
    }
}

impl IntoResponse for PushResponse {
    fn into_response(self) -> axum::response::Response {
        (self.status.status_code(), ApiJson(self)).into_response()
    }
}
