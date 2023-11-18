use anyhow::{Context, Result as AnyResult};
use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use openapi::models::PostableAlert;
use plugins_definitions::Plugin;
use push_definitions::Push;
use std::{net::SocketAddr, ops::Deref, sync::Arc};
trait Serverble: Push + Plugin {}

#[derive(Clone)]
struct ApiV2State {
    inner: Arc<ApiV2StateInner>,
}

struct ApiV2StateInner {
    plugins: Vec<Box<dyn Serverble>>,
}

impl Deref for ApiV2State {
    type Target = ApiV2StateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

async fn push(
    State(state): State<ApiV2State>,
    Json(postable_alerts): Json<Vec<PostableAlert>>,
) -> impl IntoResponse {
    for plugin in &state.plugins {
        plugin.push(&postable_alerts).await.unwrap();
    }
    "PUSH"
}

#[tokio::main]
async fn main() -> AnyResult<()> {
    let plugins: Vec<Box<dyn Serverble>> = vec![];
    for plugin in &plugins {
        plugin.initialize().await.unwrap();
    }
    let state = ApiV2State {
        inner: Arc::new(ApiV2StateInner { plugins }),
    };

    let v2 = Router::new()
        .route("/push", post(push))
        .with_state(state.clone());

    let api = Router::new().nest("/v2", v2);

    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .nest("/api", api)
        .with_state(state);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .context("Server failed")?;

    Ok(())
}
