use std::collections::BTreeMap;

use error::NewFilterPluginError;
use models::AlertmanagerPush;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

mod error;
mod impls;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum RegexActionTarget {
    Name,
    Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DropRegexAction {
    pub pattern: String,
    pub target: RegexActionTarget,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReplaceRegexAction {
    pub pattern: String,
    pub target: RegexActionTarget,
    pub replace_with: String,
    pub replacement_target: RegexActionTarget,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AddAction {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum Action {
    Drop(DropRegexAction),
    Replace(ReplaceRegexAction),
    Add(AddAction),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
/// Configuration for the Filter plugin
pub struct FilterPluginConfig {
    group_labels: Option<Vec<Action>>,
    common_labels: Option<Vec<Action>>,
    common_annotations: Option<Vec<Action>>,
    alerts_labels: Option<Vec<Action>>,
    alerts_annotations: Option<Vec<Action>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
/// Metadata for the Filter plugin
pub struct FilterPluginMeta {
    /// Name of the plugin
    pub name: String,
    /// Group of the plugin
    pub group: String,
}

/// The Filter plugin
pub struct FilterPlugin {
    /// Meta information for the plugin
    meta: FilterPluginMeta,
    /// Configuration for the plugin
    config: FilterPluginConfig,
}

impl FilterPlugin {
    pub async fn new(
        meta: FilterPluginMeta,
        config: FilterPluginConfig,
    ) -> Result<Self, NewFilterPluginError> {
        Ok(Self { meta, config })
    }

    fn filter(&self, push: &AlertmanagerPush) -> AlertmanagerPush {
        let mut push = push.clone();
        if let Some(ref actions) = self.config.group_labels {
            push.group_labels = Self::filter_btree_map(actions, push.group_labels);
        }
        if let Some(ref actions) = self.config.common_labels {
            push.common_labels = Self::filter_btree_map(actions, push.common_labels);
        }
        if let Some(ref actions) = self.config.common_annotations {
            push.common_annotations = Self::filter_btree_map(actions, push.common_annotations);
        }
        if let Some(ref actions) = self.config.alerts_labels {
            push.alerts = push
                .alerts
                .into_iter()
                .map(|alert| {
                    let mut alert = alert;
                    alert.labels = Self::filter_btree_map(actions, alert.labels);
                    alert
                })
                .collect();
        }

        push
    }

    fn filter_btree_map(
        actions: &Vec<Action>,
        mut btree_map: BTreeMap<String, String>,
    ) -> BTreeMap<String, String> {
        for action in actions {
            match action {
                Action::Drop(regex_action) => {
                    btree_map.retain(|key, value| match regex_action.target {
                        RegexActionTarget::Name => !regex::Regex::new(&regex_action.pattern)
                            .unwrap()
                            .is_match(key),
                        RegexActionTarget::Value => !regex::Regex::new(&regex_action.pattern)
                            .unwrap()
                            .is_match(value),
                    });
                }
                Action::Replace(regex_action) => {
                    btree_map = btree_map
                        .into_iter()
                        .map(|(key, value)| match regex_action.target {
                            RegexActionTarget::Name => {
                                if regex::Regex::new(&regex_action.pattern)
                                    .unwrap()
                                    .is_match(&key)
                                {
                                    match regex_action.replacement_target {
                                        RegexActionTarget::Name => {
                                            (regex_action.replace_with.clone(), value)
                                        }
                                        RegexActionTarget::Value => {
                                            (key, regex_action.replace_with.clone())
                                        }
                                    }
                                } else {
                                    (key, value)
                                }
                            }
                            RegexActionTarget::Value => {
                                if regex::Regex::new(&regex_action.pattern)
                                    .unwrap()
                                    .is_match(&value)
                                {
                                    match regex_action.replacement_target {
                                        RegexActionTarget::Name => {
                                            (regex_action.replace_with.clone(), value)
                                        }
                                        RegexActionTarget::Value => {
                                            (key, regex_action.replace_with.clone())
                                        }
                                    }
                                } else {
                                    (key, value)
                                }
                            }
                        })
                        .collect();
                }
                Action::Add(add_action) => {
                    btree_map.insert(add_action.name.clone(), add_action.value.clone());
                }
            }
        }

        btree_map
    }
}

#[cfg(test)]

mod test {
    use super::*;

    #[test]
    fn filter_group_labels() {
        let group_labels: BTreeMap<String, String> = [
            ("alertname".to_string(), "Test".to_string()),
            ("foo".to_string(), "bar".to_string()),
            ("severity".to_string(), "warning".to_string()),
            ("instance".to_string(), "localhost".to_string()),
            ("job".to_string(), "node".to_string()),
            ("replace_my_value".to_string(), "replace_me".to_string()),
            ("replace_me".to_string(), "replace_my_name".to_string()),
            ("baz".to_string(), "booz".to_string()),
        ]
        .into();

        let push = AlertmanagerPush {
            group_labels,
            ..Default::default()
        };

        let config = FilterPluginConfig {
            group_labels: Some(vec![
                Action::Drop(DropRegexAction {
                    pattern: "^foo$".to_string(),
                    target: RegexActionTarget::Name,
                }),
                Action::Drop(DropRegexAction {
                    pattern: "^warning.*".to_string(),
                    target: RegexActionTarget::Value,
                }),
                Action::Replace(ReplaceRegexAction {
                    pattern: "^inst.*".to_string(),
                    target: RegexActionTarget::Name,
                    replace_with: "instagram".to_string(),
                    replacement_target: RegexActionTarget::Name,
                }),
                Action::Replace(ReplaceRegexAction {
                    pattern: "node".to_string(),
                    target: RegexActionTarget::Value,
                    replace_with: "christmas".to_string(),
                    replacement_target: RegexActionTarget::Value,
                }),
                Action::Replace(ReplaceRegexAction {
                    pattern: "replace_my_value".to_string(),
                    target: RegexActionTarget::Name,
                    replace_with: "replaced!".to_string(),
                    replacement_target: RegexActionTarget::Value,
                }),
                Action::Replace(ReplaceRegexAction {
                    pattern: "replace_my_name".to_string(),
                    target: RegexActionTarget::Value,
                    replace_with: "replaced!".to_string(),
                    replacement_target: RegexActionTarget::Name,
                }),
                Action::Add(AddAction {
                    name: "baz".to_string(),
                    value: "baaz".to_string(),
                }),
                Action::Add(AddAction {
                    name: "test".to_string(),
                    value: "test".to_string(),
                }),
            ]),
            ..Default::default()
        };

        let plugin = FilterPlugin {
            meta: FilterPluginMeta {
                name: "test".to_string(),
                group: "test".to_string(),
            },
            config,
        };

        let push = plugin.filter(&push);

        assert_eq!(
            push.group_labels.get("alertname"),
            Some(&"Test".to_string())
        );
        assert_eq!(push.group_labels.get("foo"), None);
        assert_eq!(push.group_labels.get("severity"), None);
        assert_eq!(
            push.group_labels.get("instagram"),
            Some(&"localhost".to_string())
        );
        assert_eq!(push.group_labels.get("job"), Some(&"christmas".to_string()));
        assert_eq!(
            push.group_labels.get("replace_my_value"),
            Some(&"replaced!".to_string())
        );
        assert_eq!(
            push.group_labels.get("replaced!"),
            Some(&"replace_my_name".to_string())
        );
        assert_eq!(push.group_labels.get("baz"), Some(&"baaz".to_string()));
        assert_eq!(push.group_labels.get("test"), Some(&"test".to_string()));
    }
}
