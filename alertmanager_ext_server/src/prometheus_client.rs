use plugins_definitions::PluginMeta;
use prometheus_client::{
    encoding::{text, EncodeLabelSet},
    metrics::{counter::Counter, family::Family},
    registry::Registry,
};
use std::fmt::Error as FmtError;

#[derive(Debug, Clone, PartialEq, Eq, Hash, EncodeLabelSet)]
pub struct PushLabel {
    pub plugin_name: String,
    pub plugin_type: String,
    pub plugin_group: String,
}

impl<'a> From<PluginMeta<'a>> for PushLabel {
    fn from(plugin_meta: PluginMeta) -> Self {
        Self {
            plugin_name: plugin_meta.name.to_owned(),
            plugin_type: plugin_meta.type_.to_owned(),
            plugin_group: plugin_meta.group.to_owned(),
        }
    }
}

pub struct PromtheusClient {
    registry: Registry,
    success_push_counter: Family<PushLabel, Counter<u64>>,
    failed_push_counter: Family<PushLabel, Counter<u64>>,
}

impl PromtheusClient {
    fn new() -> Self {
        let mut registry = Registry::default();

        let success_push_counter = Family::<PushLabel, Counter<u64>>::default();
        registry.register(
            "push_success_total",
            "Total number of successful pushes",
            success_push_counter.clone(),
        );

        let failed_push_counter = Family::<PushLabel, Counter<u64>>::default();
        registry.register(
            "push_failed_total",
            "Total number of failed pushes",
            failed_push_counter.clone(),
        );

        Self {
            registry,
            success_push_counter,
            failed_push_counter,
        }
    }

    pub fn metrics(&self) -> Result<String, FmtError> {
        let mut buffer = String::new();
        text::encode(&mut buffer, &self.registry)?;
        Ok(buffer)
    }

    pub fn add_success_push(&self, label: &PushLabel) {
        self.success_push_counter.get_or_create(label).inc();
    }

    pub fn add_failed_push(&self, label: &PushLabel) {
        self.failed_push_counter.get_or_create(label).inc();
    }
}

impl Default for PromtheusClient {
    fn default() -> Self {
        Self::new()
    }
}
