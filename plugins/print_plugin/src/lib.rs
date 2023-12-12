use error::NewPrintPluginError;
use formatter::{Formatter, FormatterConfig};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

mod error;
mod impls;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
/// Configuration for the Print plugin
pub struct PrintPluginConfig {
    /// Formatting configuration
    pub formatter_config: FormatterConfig,
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
    /// Formatter
    formatter: Formatter,
}

impl PrintPlugin {
    pub async fn new(
        meta: PrintPluginMeta,
        config: PrintPluginConfig,
    ) -> Result<Self, NewPrintPluginError> {
        let formatter = Formatter::new(config.formatter_config.clone()).await?;

        Ok(Self { meta, formatter })
    }
}
