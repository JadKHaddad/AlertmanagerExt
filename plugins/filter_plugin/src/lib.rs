use error::NewFilterPluginError;
use models::AlertmanagerPush;
use plugins_definitions::Plugin;
use regex::Error as RegexError;
use regex::Regex;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::ops::Deref;
use std::str::FromStr;
use url::Url;

mod error;
mod impls;

#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegexHolder {
    #[serde_as(as = "serde_with::DisplayFromStr")]
    pub regex: Regex,
}

impl JsonSchema for RegexHolder {
    fn schema_name() -> String {
        "Regex".to_string()
    }

    fn json_schema(gen: &mut schemars::gen::SchemaGenerator) -> schemars::schema::Schema {
        String::json_schema(gen)
    }
}

impl std::fmt::Display for RegexHolder {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.regex.fmt(f)
    }
}

impl FromStr for RegexHolder {
    type Err = RegexError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            regex: Regex::new(s)?,
        })
    }
}

impl Deref for RegexHolder {
    type Target = Regex;

    fn deref(&self) -> &Self::Target {
        &self.regex
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum RegexActionTarget {
    Name,
    Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DropRegexAction {
    #[serde(flatten)]
    pub regex: RegexHolder,
    pub regex_target: RegexActionTarget,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReplaceRegexAction {
    #[serde(flatten)]
    pub regex: RegexHolder,
    pub regex_target: RegexActionTarget,
    pub replace_with: String,
    pub replacement_target: RegexActionTarget,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct AddAction {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "action")]
pub enum Action {
    Drop(DropRegexAction),
    Replace(ReplaceRegexAction),
    Add(AddAction),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
/// Configuration for the Filter plugin
pub struct FilterPluginConfig {
    pub webhook_url: Url,
    pub group_labels: Vec<Action>,
    pub common_labels: Vec<Action>,
    pub common_annotations: Vec<Action>,
    pub alerts_labels: Vec<Action>,
    pub alerts_annotations: Vec<Action>,
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

    fn add_signature(&self, push: &mut AlertmanagerPush) {
        push.common_labels
            .insert(self.name().to_string(), "filtered".to_string());
    }

    fn is_signature_present(&self, push: &AlertmanagerPush) -> bool {
        push.common_labels
            .get(self.name())
            .map(|value| value == "filtered")
            .unwrap_or(false)
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
        actions: &Vec<Action>,
        mut btree_map: BTreeMap<String, String>,
    ) -> BTreeMap<String, String> {
        for action in actions {
            match action {
                Action::Drop(regex_action) => {
                    btree_map.retain(|key, value| match regex_action.regex_target {
                        RegexActionTarget::Name => !regex_action.regex.is_match(key),
                        RegexActionTarget::Value => !regex_action.regex.is_match(value),
                    });
                }
                Action::Replace(regex_action) => {
                    btree_map = btree_map
                        .into_iter()
                        .map(|(key, value)| match regex_action.regex_target {
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

    fn create_group_labels_configs() -> FilterPluginConfig {
        FilterPluginConfig {
            webhook_url: Url::parse("http://localhost:8080").unwrap(),
            group_labels: vec![
                Action::Drop(DropRegexAction {
                    regex: RegexHolder::from_str("^foo$").unwrap(),
                    regex_target: RegexActionTarget::Name,
                }),
                Action::Drop(DropRegexAction {
                    regex: RegexHolder::from_str("^warning.*").unwrap(),
                    regex_target: RegexActionTarget::Value,
                }),
                Action::Replace(ReplaceRegexAction {
                    regex: RegexHolder::from_str("^inst.*").unwrap(),
                    regex_target: RegexActionTarget::Name,
                    replace_with: "instagram".to_string(),
                    replacement_target: RegexActionTarget::Name,
                }),
                Action::Replace(ReplaceRegexAction {
                    regex: RegexHolder::from_str("^node$").unwrap(),
                    regex_target: RegexActionTarget::Value,
                    replace_with: "christmas".to_string(),
                    replacement_target: RegexActionTarget::Value,
                }),
                Action::Replace(ReplaceRegexAction {
                    regex: RegexHolder::from_str("^replace_my_value$").unwrap(),
                    regex_target: RegexActionTarget::Name,
                    replace_with: "replaced!".to_string(),
                    replacement_target: RegexActionTarget::Value,
                }),
                Action::Replace(ReplaceRegexAction {
                    regex: RegexHolder::from_str("^replace_my_name$").unwrap(),
                    regex_target: RegexActionTarget::Value,
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
            ],
            common_labels: vec![],
            common_annotations: vec![],
            alerts_labels: vec![],
            alerts_annotations: vec![],
        }
    }

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

        let plugin = FilterPlugin {
            meta: FilterPluginMeta {
                name: "test".to_string(),
                group: "test".to_string(),
            },
            config: create_group_labels_configs(),
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

    #[ignore]
    #[test]
    fn serialize_and_print() {
        let config = create_group_labels_configs();
        let config = serde_yaml::to_string(&config).unwrap();
        println!("{}", config);
    }
}
