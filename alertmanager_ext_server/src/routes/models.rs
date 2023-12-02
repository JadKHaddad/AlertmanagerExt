use plugins_definitions::PluginMeta;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

#[derive(Debug, Clone, Serialize, JsonSchema, ToSchema)]
#[serde(rename_all = "camelCase")]
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
#[serde(rename_all = "camelCase")]
/// Query filter for plugins
///
/// This is used to filter plugins by name, group or type.
/// If none of the fields are specified, all plugins are assumed.
pub struct PluginFilterQuery {
    /// Name of the plugin
    pub name: Option<String>,
    /// Group of the plugin
    pub group: Option<String>,
    /// Type of the plugin
    pub type_: Option<String>,
}