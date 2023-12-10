use error::NewPrintPluginError;
use jinja_renderer::JinjaRenderer;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

mod error;
mod impls;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
/// The `PrintType` enum is used to specify the type of printing that should be done
pub enum PrintType {
    /// Prints as a debug string
    Debug,
    /// Prints as a pretty string
    Pretty,
    /// Prints as a json object
    Json,
    /// Prints as a yaml object
    Yaml,
    /// Prints as a jinja template
    Jinja { template: PathBuf },
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
/// Configuration for the Print plugin
pub struct PrintPluginConfig {
    /// The type of printing to do
    pub print_type: PrintType,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
/// Metadata for the Print plugin
pub struct PrintPluginMeta {
    /// Name of the plugin
    pub name: String,
    /// Group of the plugin
    pub group: String,
}

/// The Print plugin
pub struct PrintPlugin {
    /// Meta information for the plugin
    meta: PrintPluginMeta,
    /// Configuration for the plugin
    config: PrintPluginConfig,
    /// Renderer for Jinja templates
    jinja_renderer: Option<JinjaRenderer>,
}

impl PrintPlugin {
    pub async fn new(
        meta: PrintPluginMeta,
        config: PrintPluginConfig,
    ) -> Result<Self, NewPrintPluginError> {
        let jinja_renderer = match &config.print_type {
            PrintType::Jinja { template } => {
                let jinja_renderer = JinjaRenderer::new_from_file(template).await?;
                Some(jinja_renderer)
            }
            _ => None,
        };

        Ok(Self {
            meta,
            config,
            jinja_renderer,
        })
    }
}
