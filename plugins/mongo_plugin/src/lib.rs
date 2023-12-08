use anyhow::{Context, Result as AnyResult};
use database::models::{
    alert::{InsertableAlert, InsertableAlertAnnotation, InsertableAlertLabel},
    group::{
        InsertableAlertGroup, InsertableCommonAnnotation, InsertableCommonLabel,
        InsertableGroupLabel,
    },
};
use mongodb::{options::ClientOptions, Client, Collection, Database};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

mod database;
mod error;
mod impls;

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
/// Configuration for the mongo plugin
pub struct MongoPluginConfig {
    /// Connection string to the mongo database
    pub connection_string: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
/// Meta information for the mongo plugin
pub struct MongoPluginMeta {
    /// Name of the plugin
    pub name: String,
    /// Group of the plugin
    pub group: String,
}

/// Mongo plugin
///
/// Based on [`mongodb`].
pub struct MongoPlugin {
    meta: MongoPluginMeta,
    config: Option<Box<MongoPluginConfig>>,
    client: Client,
}

impl MongoPlugin {
    pub async fn new(meta: MongoPluginMeta, config: MongoPluginConfig) -> AnyResult<Self> {
        let client_options = ClientOptions::parse(&config.connection_string)
            .await
            .context("Failed to parse connection string")?;

        let client = Client::with_options(client_options).context("Failed to create client")?;

        Ok(Self {
            meta,
            config: Some(Box::new(config)),
            client,
        })
    }

    fn database(&self) -> Database {
        self.client.database("alertmanager")
    }

    fn alert_group_collection(&self) -> Collection<InsertableAlertGroup> {
        self.database().collection("alert_group")
    }

    fn group_label_collection(&self) -> Collection<InsertableGroupLabel> {
        self.database().collection("group_label")
    }

    fn common_label_collection(&self) -> Collection<InsertableCommonLabel> {
        self.database().collection("common_label")
    }

    fn common_annotation_collection(&self) -> Collection<InsertableCommonAnnotation> {
        self.database().collection("common_annotation")
    }

    fn alert_collection(&self) -> Collection<InsertableAlert> {
        self.database().collection("alert")
    }

    fn alert_label_collection(&self) -> Collection<InsertableAlertLabel> {
        self.database().collection("alert_label")
    }

    fn alert_annotation_collection(&self) -> Collection<InsertableAlertAnnotation> {
        self.database().collection("alert_annotation")
    }
}
