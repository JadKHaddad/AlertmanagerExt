use aide::{
    axum::{
        routing::{get, post},
        ApiRouter, IntoApiResponse,
    },
    openapi::{Info, OpenApi},
};
use anyhow::{Context, Result as AnyResult};
use axum::{extract::State, Extension, Json};
use models::AlermanagerPush;
use plugins_definitions::Plugin;
use push_definitions::Push;
use std::{net::SocketAddr, ops::Deref, sync::Arc};

trait Serverble: Push + Plugin {}

#[derive(Clone)]
struct ApiV1State {
    inner: Arc<ApiV1StateInner>,
}

struct ApiV1StateInner {
    plugins: Vec<Box<dyn Serverble>>,
}

impl Deref for ApiV1State {
    type Target = ApiV1StateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

async fn push(
    State(state): State<ApiV1State>,
    Json(alertmanager_push): Json<AlermanagerPush>,
) -> impl IntoApiResponse {
    for plugin in &state.plugins {
        plugin.push(&alertmanager_push).await.unwrap();
    }
    "PUSH"
}

async fn serve_api(Extension(api): Extension<OpenApi>) -> impl IntoApiResponse {
    Json(api)
}

#[tokio::main]
async fn main() -> AnyResult<()> {
    let plugins: Vec<Box<dyn Serverble>> = vec![];
    for plugin in &plugins {
        plugin.initialize().await.unwrap();
    }
    let state = ApiV1State {
        inner: Arc::new(ApiV1StateInner { plugins }),
    };

    let v1 = ApiRouter::new()
        .api_route("/push", post(push))
        .with_state(state.clone());

    let api_v1 = ApiRouter::new().nest("/v1", v1);

    let app = ApiRouter::new()
        .route("/api.json", get(serve_api))
        .nest_api_service("/api", api_v1)
        .with_state(state);

    let mut open_api = OpenApi {
        info: Info {
            title: "AlertmanagerExt".to_string(),
            ..Info::default()
        },
        ..OpenApi::default()
    };

    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    axum::Server::bind(&addr)
        .serve(
            app.finish_api(&mut open_api)
                .layer(Extension(open_api))
                .into_make_service(),
        )
        .await
        .context("Server failed")?;

    Ok(())
}
