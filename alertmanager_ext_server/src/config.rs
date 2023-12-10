use file_plugin::{FilePluginConfig, FilePluginMeta};
use filter_plugin::{FilterPluginConfig, FilterPluginMeta};
use mongo_plugin::{MongoPluginConfig, MongoPluginMeta};
use postgres_plugin::{PostgresPluginConfig, PostgresPluginMeta};
use postgres_sea_plugin::{PostgresSeaPluginConfig, PostgresSeaPluginMeta};
use postgres_x_plugin::{PostgresXPluginConfig, PostgresXPluginMeta};
use print_plugin::{PrintPluginConfig, PrintPluginMeta};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sqlite_plugin::{SqlitePluginConfig, SqlitePluginMeta};
use std::{
    net::{Ipv4Addr, SocketAddr},
    path::Path,
    str::FromStr,
};
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum ConfigFromYamlFileError {
    #[error("Failed to read config file: {0}")]
    IoError(
        #[source]
        #[from]
        tokio::io::Error,
    ),
    #[error("Failed to parse config file: {0}")]
    YamlError(
        #[source]
        #[from]
        serde_yaml::Error,
    ),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Config {
    pub server: ServerConfig,
    pub plugins: Option<PluginsConfig>,
}

impl Config {
    pub async fn new_from_yaml_file(
        path: impl AsRef<Path>,
    ) -> Result<Self, ConfigFromYamlFileError> {
        let config = tokio::fs::read_to_string(path).await?;
        let config = serde_yaml::from_str::<Self>(&config)?;
        Ok(config)
    }

    pub async fn new_from_yaml_str(config: &str) -> Result<Self, ConfigFromYamlFileError> {
        let config = serde_yaml::from_str::<Self>(config)?;
        Ok(config)
    }

    pub fn addr(&self) -> SocketAddr {
        SocketAddr::new(
            Ipv4Addr::from(self.server.host.clone()).into(),
            self.server.port,
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ServerConfig {
    pub host: LocalhostOrIpv4Addr,
    pub port: u16,
}

#[derive(JsonSchema)]
#[serde_with::serde_as]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
#[serde(untagged)]
pub enum LocalhostOrIpv4Addr {
    Ipv4Addr(Ipv4Addr),
    Localhost(#[serde_as(as = "serde_with::DisplayFromStr")] Localhost),
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq, Hash)]
pub struct Localhost;

impl std::fmt::Display for Localhost {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "localhost")
    }
}

impl FromStr for Localhost {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "localhost" {
            return Ok(Localhost);
        }
        Err(format!("{} is not localhost", s))
    }
}

impl From<LocalhostOrIpv4Addr> for Ipv4Addr {
    fn from(addr: LocalhostOrIpv4Addr) -> Self {
        match addr {
            LocalhostOrIpv4Addr::Ipv4Addr(ipv4) => ipv4,
            LocalhostOrIpv4Addr::Localhost(_) => Ipv4Addr::new(127, 0, 0, 1),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PluginsConfig {
    pub file_plugin: Option<Vec<FilePluginFromFileConfig>>,
    pub filter_plugin: Option<Vec<FilterPluginFromFileConfig>>,
    pub mongo_plugin: Option<Vec<MongoPluginFromFileConfig>>,
    pub postgres_plugin: Option<Vec<PostgresPluginFromFileConfig>>,
    pub postgres_sea_plugin: Option<Vec<PostgresSeaPluginFromFileConfig>>,
    pub postgres_x_plugin: Option<Vec<PostgresXPluginFromFileConfig>>,
    pub print_plugin: Option<Vec<PrintPluginFromFileConfig>>,
    pub sqlite_plugin: Option<Vec<SqlitePluginFromFileConfig>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FilterPluginFromFileConfig {
    pub meta: FilterPluginMeta,
    pub config: FilterPluginConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct FilePluginFromFileConfig {
    pub meta: FilePluginMeta,
    pub config: FilePluginConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct MongoPluginFromFileConfig {
    pub meta: MongoPluginMeta,
    pub config: MongoPluginConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PostgresPluginFromFileConfig {
    pub meta: PostgresPluginMeta,
    pub config: PostgresPluginConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PostgresSeaPluginFromFileConfig {
    pub meta: PostgresSeaPluginMeta,
    pub config: PostgresSeaPluginConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PostgresXPluginFromFileConfig {
    pub meta: PostgresXPluginMeta,
    pub config: PostgresXPluginConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PrintPluginFromFileConfig {
    pub meta: PrintPluginMeta,
    pub config: PrintPluginConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SqlitePluginFromFileConfig {
    pub meta: SqlitePluginMeta,
    pub config: SqlitePluginConfig,
}

#[cfg(test)]
mod test {
    use super::*;

    #[ignore]
    #[test]
    fn serialize_config_localhost_and_print() {
        let config = Config {
            server: ServerConfig {
                host: LocalhostOrIpv4Addr::Localhost(Localhost),
                port: 8080,
            },
            plugins: None,
        };
        let config = serde_json::to_string_pretty(&config).expect("failed to serialize config");
        println!("{}", config);
    }

    #[ignore]
    #[test]
    fn serialize_config_ipv4_and_print() {
        let config = Config {
            server: ServerConfig {
                host: LocalhostOrIpv4Addr::Ipv4Addr(Ipv4Addr::new(10, 12, 3, 1)),
                port: 8080,
            },
            plugins: None,
        };
        let config = serde_json::to_string_pretty(&config).expect("failed to serialize config");
        println!("{}", config);
    }

    #[ignore]
    #[test]
    fn serialize_yaml_with_plugins() {
        let config = Config {
            server: ServerConfig {
                host: LocalhostOrIpv4Addr::Ipv4Addr(Ipv4Addr::new(10, 12, 3, 1)),
                port: 8080,
            },
            plugins: Some(PluginsConfig {
                filter_plugin: Some(vec![FilterPluginFromFileConfig {
                    meta: FilterPluginMeta {
                        name: "filter_plugin".to_string(),
                        group: "filter".to_string(),
                    },
                    config: FilterPluginConfig {
                        webhook_url: url::Url::parse("http://localhost:8080").unwrap(),
                        group_labels: vec![],
                        common_labels: vec![],
                        common_annotations: vec![],
                        alerts_labels: vec![],
                        alerts_annotations: vec![],
                    },
                }]),
                file_plugin: Some(vec![
                    FilePluginFromFileConfig {
                        meta: FilePluginMeta {
                            name: "file_plugin".to_string(),
                            group: "file".to_string(),
                        },
                        config: FilePluginConfig {
                            dir_path: "/tmp".into(),
                            file_type: file_plugin::FileType::Json,
                        },
                    },
                    FilePluginFromFileConfig {
                        meta: FilePluginMeta {
                            name: "file_plugin_2".to_string(),
                            group: "file".to_string(),
                        },
                        config: FilePluginConfig {
                            dir_path: "/tmp".into(),
                            file_type: file_plugin::FileType::Json,
                        },
                    },
                ]),
                print_plugin: Some(vec![PrintPluginFromFileConfig {
                    meta: PrintPluginMeta {
                        name: "print_plugin".to_string(),
                        group: "print".to_string(),
                    },
                    config: PrintPluginConfig {
                        print_type: print_plugin::PrintType::Json,
                    },
                }]),
                mongo_plugin: None,
                postgres_plugin: None,
                postgres_sea_plugin: None,
                postgres_x_plugin: None,
                sqlite_plugin: None,
            }),
        };
        let config = serde_yaml::to_string(&config).expect("failed to serialize config");
        println!("{}", config);
    }

    #[test]
    fn deserialize_config_localhost() {
        let config = r#"
        {
            "server": {
                "host": "localhost",
                "port": 8080
            }
        }
        "#;
        let config: Config = serde_json::from_str(config).expect("failed to deserialize config");
        assert_eq!(
            config.server.host,
            LocalhostOrIpv4Addr::Localhost(Localhost)
        );
        assert_eq!(config.server.port, 8080);
    }

    #[test]
    fn deserialize_config_ipv4() {
        let config = r#"
        {
            "server": {
                "host": "10.12.3.1",
                "port": 8080
            }
        }
        "#;

        let config: Config = serde_json::from_str(config).expect("failed to deserialize config");
        assert_eq!(
            config.server.host,
            LocalhostOrIpv4Addr::Ipv4Addr(Ipv4Addr::new(10, 12, 3, 1))
        );
        assert_eq!(config.server.port, 8080);
    }
}
