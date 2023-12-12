use error::{DirError, FormatError, NewFilePluginError};
use jinja_renderer::JinjaRenderer;
use models::AlertmanagerPush;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

mod error;
mod impls;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "type")]
/// The type of file to write
pub enum FileType {
    Json,
    Yaml,
    Jinja { template: PathBuf },
}

impl FileType {
    fn extension(&self) -> &'static str {
        match self {
            Self::Json => "json",
            Self::Yaml => "yaml",
            Self::Jinja { .. } => "txt",
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
    /// Renderer for Jinja templates
    jinja_renderer: Option<JinjaRenderer>,
}

impl FilePlugin {
    pub async fn new(
        meta: FilePluginMeta,
        config: FilePluginConfig,
    ) -> Result<Self, NewFilePluginError> {
        let jinja_renderer = match &config.file_type {
            FileType::Jinja { template } => {
                let jinja_renderer = JinjaRenderer::new_from_file(template).await?;
                Some(jinja_renderer)
            }
            _ => None,
        };

        Ok(Self {
            meta,
            config,
            jinja_renderer,
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

    fn format(&self, alertmanager_push: &AlertmanagerPush) -> Result<String, FormatError> {
        let file_type = &self.config.file_type;

        match file_type {
            FileType::Json => Ok(serde_json::to_string(alertmanager_push)?),
            FileType::Yaml => Ok(serde_yaml::to_string(alertmanager_push)?),
            FileType::Jinja { .. } => {
                let jinja_renderer = self
                    .jinja_renderer
                    .as_ref()
                    .ok_or(FormatError::JinjaUninitialized)?;
                Ok(jinja_renderer.render(alertmanager_push)?)
            }
        }
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

#[cfg(test)]
mod test {
    use super::*;

    #[ignore]
    #[test]
    fn serialize_file_type() {
        let file_type = FileType::Json;
        let file_type = serde_yaml::to_string(&file_type).expect("failed to serialize file type");
        println!("{}", file_type);
        let file_type = FileType::Jinja {
            template: PathBuf::from("template.j2"),
        };
        let file_type = serde_yaml::to_string(&file_type).expect("failed to serialize file type");
        println!("{}", file_type);
    }
}
