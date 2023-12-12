use error::{DirError, NewFilePluginError};
use formatter::{Formatter, FormatterConfig};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

mod error;
mod impls;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
/// Configuration for the File plugin
pub struct FilePluginConfig {
    /// The path to the directory where the files will be stored
    pub dir_path: PathBuf,
    /// The type of file to write
    pub extension: String,
    /// Formatting configuration
    pub formatter_config: FormatterConfig,
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
    /// Formatter
    formatter: Formatter,
}

impl FilePlugin {
    pub async fn new(
        meta: FilePluginMeta,
        config: FilePluginConfig,
    ) -> Result<Self, NewFilePluginError> {
        let formatter = Formatter::new(config.formatter_config.clone()).await?;

        Ok(Self {
            meta,
            config,
            formatter,
        })
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
}
