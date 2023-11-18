use axum::{
    extract::State,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use openapi::models::PostableAlert;
use plugins_definitions::Plugin;
use push_definitions::Push;
use std::{ops::Deref, sync::Arc};

trait Serverble: Push + Plugin {}

#[derive(Clone)]
struct ServerState {
    inner: Arc<StateInner>,
}

struct StateInner {
    plugins: Vec<Box<dyn Serverble>>,
}

impl Deref for ServerState {
    type Target = StateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

async fn push(
    State(state): State<ServerState>,
    Json(postable_alerts): Json<Vec<PostableAlert>>,
) -> impl IntoResponse {
    for plugin in &state.plugins {
        plugin.push(&postable_alerts).await.unwrap();
    }
    "PUSH"
}

#[tokio::main]
async fn main() {
    let plugins: Vec<Box<dyn Serverble>> = vec![];
    for plugin in &plugins {
        plugin.initialize().await.unwrap();
    }
    let state = ServerState {
        inner: Arc::new(StateInner { plugins }),
    };

    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/push", post(push))
        .with_state(state);

    // run it with hyper on localhost:3000
    axum::Server::bind(&"0.0.0.0:3000".parse().unwrap())
        .serve(app.into_make_service())
        .await
        .unwrap();
}
