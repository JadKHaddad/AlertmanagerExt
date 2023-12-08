use error::{DirError, SerializeError};
use models::AlertmanagerPush;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

mod error;
mod impls;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
/// The type of file to write
pub enum FileType {
    Json,
    Yaml,
}

impl FileType {
    fn extension(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Yaml => "yaml",
        }
    }

    fn function<T>(&self) -> for<'b> fn(&'b T) -> Result<String, SerializeError>
    where
        T: serde::Serialize,
    {
        match self {
            Self::Json => |t: &T| {
                serde_json::to_string(t).map_err(|error| SerializeError {
                    reason: error.to_string(),
                })
            },
            Self::Yaml => |t: &T| {
                serde_yaml::to_string(t).map_err(|error| SerializeError {
                    reason: error.to_string(),
                })
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
/// Configuration for the File plugin
pub struct FilePluginConfig {
    /// The path to the directory where the files will be stored
    pub dir_path: PathBuf,
    /// The type of file to write
    pub file_type: FileType,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
/// Metadata for the File plugin
pub struct FilePluginMeta {
    /// Name of the plugin
    pub name: String,
    /// Group of the plugin
    pub group: String,
}

/// The File plugin
///
/// Based on [`tokio::fs`]
pub struct FilePlugin {
    /// Meta information for the plugin
    meta: FilePluginMeta,
    /// Configuration for the plugin
    config: FilePluginConfig,
}

impl FilePlugin {
    pub fn new(meta: FilePluginMeta, config: FilePluginConfig) -> Self {
        Self { meta, config }
    }

    fn dir_exists(&self) -> Result<(), DirError> {
        let dir_path = &self.config.dir_path;

        if !dir_path.exists() {
            return Err(DirError::DoesNotExist {
                path: dir_path.to_path_buf(),
            });
        }

        if !dir_path.is_dir() {
            return Err(DirError::NotADirectory {
                path: dir_path.to_path_buf(),
            });
        }

        Ok(())
    }

    fn serialize(&self, alertmanager_push: &AlertmanagerPush) -> Result<String, SerializeError> {
        let file_type = &self.config.file_type;

        (file_type.function())(alertmanager_push)
    }

    fn decide_file_path(&self, alertmanager_push: &AlertmanagerPush) -> PathBuf {
        let dir_path = &self.config.dir_path;
        let file_type = &self.config.file_type;

        let mut file_path = dir_path.clone();
        file_path.push(format!(
            "{}.{}",
            alertmanager_push.group_key,
            file_type.extension()
        ));

        file_path
    }
}
