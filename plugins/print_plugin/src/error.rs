use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum NewPrintPluginError {
    #[error("Failed to create jinja renderer: {0}")]
    JinjaRenderer(
        #[source]
        #[from]
        jinja_renderer::NewJinjaRendererError,
    ),
}

#[derive(ThisError, Debug)]
#[error("Failed to convert to string: {reason}")]
pub enum ToStringError {
    #[error("Failed to convert to json: {0}")]
    Json(
        #[source]
        #[from]
        serde_json::Error,
    ),
    #[error("Failed to convert to yaml: {0}")]
    Yaml(
        #[source]
        #[from]
        serde_yaml::Error,
    ),
    #[error("Failed to render template: {0}")]
    Jinja(
        #[source]
        #[from]
        jinja_renderer::RenderError,
    ),
    #[error("Other error: {reason}")]
    Other { reason: String },
}
