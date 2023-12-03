use async_trait::async_trait;
use error::{DirError, SerializeError, WriteError};
use models::AlertmanagerPush;
use plugins_definitions::{HealthError, Plugin, PluginMeta};
use push_definitions::{InitializeError, Push, PushError};
use std::path::{Path, PathBuf};

mod error;

/// The type of file to write
pub enum FileType {
    JSON,
    YAML,
}

impl FileType {
    fn extension(&self) -> &'static str {
        match self {
            Self::JSON => "json",
            Self::YAML => "yaml",
        }
    }

    fn function<T>(&self) -> for<'b> fn(&'b T) -> Result<String, SerializeError>
    where
        T: serde::Serialize,
    {
        match self {
            Self::JSON => |t: &T| {
                serde_json::to_string(t).map_err(|error| SerializeError {
                    reason: error.to_string(),
                })
            },
            Self::YAML => |t: &T| {
                serde_yaml::to_string(t).map_err(|error| SerializeError {
                    reason: error.to_string(),
                })
            },
        }
    }
}

/// Configuration for the File plugin
pub struct FilePluginConfig {
    /// The path to the directory where the files will be stored
    pub dir_path: PathBuf,
    /// The type of file to write
    pub file_type: FileType,
}

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

    async fn write_file(
        path: impl AsRef<Path>,
        contents: impl AsRef<[u8]>,
    ) -> Result<(), WriteError> {
        tokio::fs::write(path, contents)
            .await
            .map_err(|error| WriteError::Write { error })
    }
}

#[async_trait]
impl Plugin for FilePlugin {
    fn meta(&self) -> PluginMeta {
        PluginMeta {
            name: &self.meta.name,
            type_: "file",
            group: &self.meta.group,
        }
    }

    #[tracing::instrument(name = "health", skip(self), fields(name = %self.name(), group = %self.group(), type_ = %self.type_()))]
    async fn health(&self) -> Result<(), HealthError> {
        tracing::trace!("Checking health.");

        self.dir_exists().map_err(|error| HealthError {
            reason: error.to_string(),
        })?;

        tracing::trace!("Successfully checked health.");
        Ok(())
    }
}

#[async_trait]
impl Push for FilePlugin {
    #[tracing::instrument(name = "push_initialize", skip(self), fields(name = %self.name(), group = %self.group(), type_ = %self.type_()))]
    async fn initialize(&mut self) -> Result<(), InitializeError> {
        tracing::trace!("Initializing.");

        self.dir_exists().map_err(|error| InitializeError {
            reason: error.to_string(),
        })?;

        tracing::trace!("Successfully initialized.");
        Ok(())
    }

    #[tracing::instrument(name = "push_alert", skip_all, fields(name = %self.name(), group = %self.group(), type_ = %self.type_()))]
    async fn push_alert(&self, alertmanager_push: &AlertmanagerPush) -> Result<(), PushError> {
        tracing::trace!("Pushing.");

        let file_path = self.decide_file_path(alertmanager_push);
        let contents = self
            .serialize(alertmanager_push)
            .map_err(|error| PushError {
                reason: error.to_string(),
            })?;

        Self::write_file(file_path, contents)
            .await
            .map_err(|error| PushError {
                reason: error.to_string(),
            })?;

        tracing::trace!("Successfully pushed.");
        Ok(())
    }
}
