use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::{net::Ipv4Addr, path::Path, str::FromStr};
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
    pub plugins: PluginsConfig,
}

impl Config {
    pub async fn new_from_yaml_file(
        path: impl AsRef<Path>,
    ) -> Result<Self, ConfigFromYamlFileError> {
        let config = tokio::fs::read_to_string(path).await?;
        let config = serde_yaml::from_str::<Self>(&config)?;
        Ok(config)
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
pub struct PluginsConfig {}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PluginMetadata {
    pub name: String,
    pub group: String,
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
            plugins: PluginsConfig {},
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
            plugins: PluginsConfig {},
        };
        let config = serde_json::to_string_pretty(&config).expect("failed to serialize config");
        println!("{}", config);
    }

    #[test]
    fn deserialize_config_localhost() {
        let config = r#"
        {
            "server": {
                "host": "localhost",
                "port": 8080
            },
            "plugins": {}
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
            },
            "plugins": {}
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
