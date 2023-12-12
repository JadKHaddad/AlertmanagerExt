use std::path::PathBuf;

use jinja_renderer::JinjaRenderer;
use models::AlertmanagerPush;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum NewFormatterError {
    #[error("Failed to create jinja renderer: {0}")]
    JinjaRenderer(
        #[source]
        #[from]
        jinja_renderer::NewJinjaRendererError,
    ),
}

#[derive(ThisError, Debug)]
pub enum FormatError {
    #[error("Failed to convert to json: {0}")]
    Json(
        #[source]
        #[from]
        serde_json::Error,
    ),
    #[error("Failed to convert to yaml: {0}")]
    Yaml(
        #[source]
        #[from]
        serde_yaml::Error,
    ),
    #[error("Failed to render template: {0}")]
    JinjaRender(
        #[source]
        #[from]
        jinja_renderer::RenderError,
    ),
    #[error("Jinja renderer not initialized")]
    JinjaUninitialized,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
pub enum FormatType {
    /// Debug string
    Debug,
    /// Pretty string
    Pretty,
    /// Json object
    Json,
    /// Yaml object
    Yaml,
    /// Jinja template
    Jinja { template: PathBuf },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FormatterConfig {
    pub format_type: FormatType,
}

pub struct Formatter {
    config: FormatterConfig,
    jinja_renderer: Option<JinjaRenderer>,
}

impl Formatter {
    pub async fn new(config: FormatterConfig) -> Result<Self, NewFormatterError> {
        let jinja_renderer = match &config.format_type {
            FormatType::Jinja { template } => {
                let jinja_renderer = JinjaRenderer::new_from_file(template).await?;
                Some(jinja_renderer)
            }
            _ => None,
        };

        Ok(Self {
            config,
            jinja_renderer,
        })
    }

    pub fn format(&self, alertmanager_push: &AlertmanagerPush) -> Result<String, FormatError> {
        let formatted = match self.config.format_type {
            FormatType::Debug => format!("{:?}", alertmanager_push),
            FormatType::Pretty => format!("{:#?}", alertmanager_push),
            FormatType::Json => serde_json::to_string(alertmanager_push)?,
            FormatType::Yaml => serde_yaml::to_string(alertmanager_push)?,
            FormatType::Jinja { .. } => {
                let jinja_renderer = self
                    .jinja_renderer
                    .as_ref()
                    .ok_or(FormatError::JinjaUninitialized)?;

                jinja_renderer.render(alertmanager_push)?
            }
        };

        Ok(formatted)
    }
}
