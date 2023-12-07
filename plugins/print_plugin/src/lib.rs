mod impls;

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
}

/// Configuration for the Print plugin
pub struct PrintPluginConfig {
    /// The type of printing to do
    pub print_type: PrintType,
}

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
}

impl PrintPlugin {
    pub fn new(meta: PrintPluginMeta, config: PrintPluginConfig) -> Self {
        Self { meta, config }
    }
}
