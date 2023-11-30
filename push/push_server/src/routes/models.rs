use plugins_definitions::{OwnedPluginMeta, PluginMeta};
use schemars::JsonSchema;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, Serialize, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase")]
/// Meta information about a plugin
pub struct PluginResponseMeta {
    /// Name of the plugin
    pub plugin_name: String,
    /// Type of the plugin, if found
    pub plugin_type: Option<&'static str>,
    /// Group of the plugin, if found
    pub plugin_group: Option<String>,
}

impl PluginResponseMeta {
    /// Helper function
    ///
    /// Creates a [`PluginResponseMeta`] for a plugin that was not found
    pub fn not_found(plugin_name: String) -> Self {
        Self {
            plugin_name,
            plugin_type: None,
            plugin_group: None,
        }
    }
}

impl<'a> From<PluginMeta<'a>> for PluginResponseMeta {
    fn from(plugin_meta: PluginMeta) -> Self {
        Self::from(OwnedPluginMeta::from(plugin_meta))
    }
}

impl From<OwnedPluginMeta> for PluginResponseMeta {
    fn from(plugin_meta: OwnedPluginMeta) -> Self {
        Self {
            plugin_name: plugin_meta.name,
            plugin_type: Some(plugin_meta.type_),
            plugin_group: Some(plugin_meta.group),
        }
    }
}
