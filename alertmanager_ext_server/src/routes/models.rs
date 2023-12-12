use plugins_definitions::PluginMeta;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Serialize, JsonSchema, ToSchema)]
/// Meta information about a plugin
pub struct PluginResponseMeta {
    /// Name of the plugin
    pub plugin_name: String,
    /// Type of the plugin
    pub plugin_type: String,
    /// Group of the plugin
    pub plugin_group: String,
}

impl<'a> From<PluginMeta<'a>> for PluginResponseMeta {
    fn from(plugin_meta: PluginMeta) -> Self {
        Self {
            plugin_name: plugin_meta.name.to_owned(),
            plugin_type: plugin_meta.type_.to_owned(),
            plugin_group: plugin_meta.group.to_owned(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, IntoParams, ToSchema)]
/// Query filter for plugins
///
/// This is used to filter plugins by name, group or type.
pub struct PluginFilterQuery {
    pub filter: Option<String>,
}
