use minijinja::{context, Environment};
use models::AlertmanagerPush;
use std::path::Path;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum RendererFromFileError {
    #[error("Failed to read file: {0}")]
    IoError(
        #[source]
        #[from]
        tokio::io::Error,
    ),
}

pub struct Renderer {
    template: String,
}

impl Renderer {
    pub async fn new_from_file(file: impl AsRef<Path>) -> Result<Self, RendererFromFileError> {
        let template = tokio::fs::read_to_string(file).await?;
        Ok(Self { template })
    }

    pub async fn new_from_str(template: impl Into<String>) -> Result<Self, RendererFromFileError> {
        Ok(Self {
            template: template.into(),
        })
    }

    pub fn render(&self, push: &AlertmanagerPush) -> Result<String, minijinja::Error> {
        let mut env = Environment::new();
        minijinja_contrib::add_to_environment(&mut env);
        env.add_template("push", &self.template)?;
        let template = env.get_template("push")?;
        let context = context! {
            push => push
        };
        template.render(context)
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
        let renderer = Renderer::new_from_str(include_str!("../template.j2"))
            .await
            .expect("failed to create renderer");

        let rendered = renderer.render(&push).expect("failed to render");
        println!("{}", rendered);
    }

    #[ignore]
    #[tokio::test]
    async fn render_from_file() {
        let push = generate_random_alertmanager_pushes(1).pop().unwrap();
        let renderer = Renderer::new_from_file(Path::new("template.j2"))
            .await
            .expect("failed to create renderer");

        let rendered = renderer.render(&push).expect("failed to render");
        println!("{}", rendered);
    }
}
