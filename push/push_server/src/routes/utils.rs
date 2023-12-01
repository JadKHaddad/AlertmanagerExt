use super::models::PluginFilterQuery;
use crate::traits::PushAndPlugin;
use std::sync::Arc;

pub fn filter_plugins<'a>(
    plugins: &'a [Arc<dyn PushAndPlugin>],
    filter_query: &PluginFilterQuery,
) -> Vec<&'a Arc<dyn PushAndPlugin>> {
    plugins
        .iter()
        .filter(|p| {
            let meta = p.meta();

            let group_matches = filter_query
                .group
                .as_ref()
                .map(|group| meta.group == group)
                .unwrap_or(true);

            let name_matches = filter_query
                .name
                .as_ref()
                .map(|name| meta.name == name)
                .unwrap_or(true);

            let type_matches = filter_query
                .type_
                .as_ref()
                .map(|type_| meta.type_ == type_)
                .unwrap_or(true);

            group_matches && name_matches && type_matches
        })
        .collect()
}
