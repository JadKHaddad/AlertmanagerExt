use minijinja::{context, Environment};
use models::AlertmanagerPush;
use std::path::Path;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum NewJinjaRendererError {
    #[error("Failed to read file: {0}")]
    IoError(
        #[source]
        #[from]
        tokio::io::Error,
    ),
    #[error("Failed to configure template: {0}")]
    MinijinjaError(
        #[source]
        #[from]
        minijinja::Error,
    ),
}

#[derive(ThisError, Debug)]
pub enum RenderError {
    #[error("Failed to render template: {0}")]
    MinijinjaError(
        #[source]
        #[from]
        minijinja::Error,
    ),
}

pub struct JinjaRenderer {
    env: Environment<'static>,
}

impl JinjaRenderer {
    pub async fn new_from_str(template: impl Into<String>) -> Result<Self, NewJinjaRendererError> {
        let template = template.into();
        let mut env = Environment::new();
        minijinja_contrib::add_to_environment(&mut env);
        env.add_template_owned("push", template)?;

        Ok(Self { env })
    }

    pub async fn new_from_file(file: impl AsRef<Path>) -> Result<Self, NewJinjaRendererError> {
        let template = tokio::fs::read_to_string(file).await?;
        Self::new_from_str(template).await
    }

    pub fn render(&self, push: &AlertmanagerPush) -> Result<String, RenderError> {
        let template = self.env.get_template("push")?;
        let context = context! {
            push => push
        };
        Ok(template.render(context)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use random_models_generator::generate_random_alertmanager_pushes;

    #[ignore]
    #[tokio::test]
    async fn render_from_str() {
        let push = generate_random_alertmanager_pushes(1).pop().unwrap();
        let renderer = JinjaRenderer::new_from_str(include_str!("../example-template.j2"))
            .await
            .expect("failed to create renderer");

        let rendered = renderer.render(&push).expect("failed to render");
        println!("{}", rendered);
    }

    #[ignore]
    #[tokio::test]
    async fn render_from_file() {
        let push = generate_random_alertmanager_pushes(1).pop().unwrap();
        let renderer = JinjaRenderer::new_from_file(Path::new("example-template.j2"))
            .await
            .expect("failed to create renderer");

        let rendered = renderer.render(&push).expect("failed to render");
        println!("{}", rendered);
    }
}
