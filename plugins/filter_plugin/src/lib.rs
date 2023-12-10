use error::NewFilterPluginError;
use models::AlertmanagerPush;
use regex::Error as RegexError;
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

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

#[derive(Debug, Clone)]
pub struct PreparedDropRegexAction {
    pub regex: Regex,
    pub target: RegexActionTarget,
}

impl TryFrom<DropRegexAction> for PreparedDropRegexAction {
    type Error = RegexError;

    fn try_from(value: DropRegexAction) -> Result<Self, Self::Error> {
        Ok(Self {
            regex: Regex::new(&value.pattern)?,
            target: value.target,
        })
    }
}

#[derive(Debug, Clone)]
pub struct PreparedReplaceRegexAction {
    pub regex: Regex,
    pub target: RegexActionTarget,
    pub replace_with: String,
    pub replacement_target: RegexActionTarget,
}

impl TryFrom<ReplaceRegexAction> for PreparedReplaceRegexAction {
    type Error = RegexError;

    fn try_from(value: ReplaceRegexAction) -> Result<Self, Self::Error> {
        Ok(Self {
            regex: Regex::new(&value.pattern)?,
            target: value.target,
            replace_with: value.replace_with,
            replacement_target: value.replacement_target,
        })
    }
}

#[derive(Debug, Clone)]
pub enum PreparedAction {
    Drop(PreparedDropRegexAction),
    Replace(PreparedReplaceRegexAction),
    Add(AddAction),
}

impl TryFrom<Action> for PreparedAction {
    type Error = RegexError;

    fn try_from(value: Action) -> Result<Self, Self::Error> {
        match value {
            Action::Drop(drop_regex_action) => Ok(PreparedAction::Drop(
                PreparedDropRegexAction::try_from(drop_regex_action)?,
            )),
            Action::Replace(replace_regex_action) => Ok(PreparedAction::Replace(
                PreparedReplaceRegexAction::try_from(replace_regex_action)?,
            )),
            Action::Add(add_action) => Ok(PreparedAction::Add(add_action)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PreparedFilterPluginConfig {
    group_labels: Vec<PreparedAction>,
    common_labels: Vec<PreparedAction>,
    common_annotations: Vec<PreparedAction>,
    alerts_labels: Vec<PreparedAction>,
    alerts_annotations: Vec<PreparedAction>,
}

impl PreparedFilterPluginConfig {
    fn try_from_opt_vec_action(
        value: Option<Vec<Action>>,
    ) -> Result<Vec<PreparedAction>, RegexError> {
        Ok(value
            .map(|actions| {
                actions
                    .into_iter()
                    .map(PreparedAction::try_from)
                    .collect::<Result<Vec<PreparedAction>, RegexError>>()
            })
            .transpose()?
            .unwrap_or_default())
    }
}

impl TryFrom<FilterPluginConfig> for PreparedFilterPluginConfig {
    type Error = RegexError;

    fn try_from(value: FilterPluginConfig) -> Result<Self, Self::Error> {
        let group_labels = PreparedFilterPluginConfig::try_from_opt_vec_action(value.group_labels)?;
        let common_labels =
            PreparedFilterPluginConfig::try_from_opt_vec_action(value.common_labels)?;
        let common_annotations =
            PreparedFilterPluginConfig::try_from_opt_vec_action(value.common_annotations)?;
        let alerts_labels =
            PreparedFilterPluginConfig::try_from_opt_vec_action(value.alerts_labels)?;
        let alerts_annotations =
            PreparedFilterPluginConfig::try_from_opt_vec_action(value.alerts_annotations)?;

        Ok(Self {
            group_labels,
            common_labels,
            common_annotations,
            alerts_labels,
            alerts_annotations,
        })
    }
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
    config: PreparedFilterPluginConfig,
}

impl FilterPlugin {
    pub async fn new(
        meta: FilterPluginMeta,
        config: FilterPluginConfig,
    ) -> Result<Self, NewFilterPluginError> {
        let prepared_conmfig = PreparedFilterPluginConfig::try_from(config)?;
        Ok(Self {
            meta,
            config: prepared_conmfig,
        })
    }

    fn filter(&self, push: &AlertmanagerPush) -> AlertmanagerPush {
        let mut push = push.clone();

        push.group_labels = Self::filter_btree_map(&self.config.group_labels, push.group_labels);
        push.common_labels = Self::filter_btree_map(&self.config.common_labels, push.common_labels);
        push.common_annotations =
            Self::filter_btree_map(&self.config.common_annotations, push.common_annotations);

        push.alerts = push
            .alerts
            .into_iter()
            .map(|alert| {
                let mut alert = alert;
                alert.labels = Self::filter_btree_map(&self.config.alerts_labels, alert.labels);
                alert.annotations =
                    Self::filter_btree_map(&self.config.alerts_annotations, alert.annotations);
                alert
            })
            .collect();

        push
    }

    fn filter_btree_map(
        actions: &Vec<PreparedAction>,
        mut btree_map: BTreeMap<String, String>,
    ) -> BTreeMap<String, String> {
        for action in actions {
            match action {
                PreparedAction::Drop(regex_action) => {
                    btree_map.retain(|key, value| match regex_action.target {
                        RegexActionTarget::Name => !regex_action.regex.is_match(key),
                        RegexActionTarget::Value => !regex_action.regex.is_match(value),
                    });
                }
                PreparedAction::Replace(regex_action) => {
                    btree_map = btree_map
                        .into_iter()
                        .map(|(key, value)| match regex_action.target {
                            RegexActionTarget::Name => {
                                if regex_action.regex.is_match(&key) {
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
                                if regex_action.regex.is_match(&value) {
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
                PreparedAction::Add(add_action) => {
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
            config: PreparedFilterPluginConfig::try_from(config)
                .expect("I guess i'm bad at regex :D"),
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
