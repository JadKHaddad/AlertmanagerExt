use anyhow::{Context, Result as AnyResult};
use async_trait::async_trait;
use diesel::{Connection, PgConnection};
use diesel_async::{pooled_connection::AsyncDieselConnectionManager, AsyncPgConnection};
use diesel_migrations::MigrationHarness;
use diesel_migrations::{embed_migrations, EmbeddedMigrations};
use models::AlermanagerPush;
use plugins_definitions::{InitializeError, Plugin};
use push_definitions::{Push, PushError};
use tokio::task::JoinHandle;

mod database;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

type Pool = bb8::Pool<AsyncDieselConnectionManager<AsyncPgConnection>>;

pub struct PostgresPlugin {
    connection_string: String,
    pool: Pool,
}

impl PostgresPlugin {
    pub async fn new(connection_string: String) -> AnyResult<Self> {
        let manager = AsyncDieselConnectionManager::new(connection_string.clone());
        let pool = bb8::Pool::builder()
            .max_size(15)
            .build(manager)
            .await
            .context("Failed to create pool.")?;

        Ok(Self {
            connection_string,
            pool,
        })
    }
}

#[async_trait]
impl Plugin for PostgresPlugin {
    fn name(&self) -> &'static str {
        "Postgres"
    }

    #[tracing::instrument(name = "PostgresPlugin initialize", skip_all)]
    async fn initialize(&self) -> Result<(), InitializeError> {
        tracing::trace!("Initializing.");
        let connection_string = self.connection_string.clone();
        let handle: JoinHandle<AnyResult<()>> = tokio::task::spawn_blocking(move || {
            let mut conn = PgConnection::establish(&connection_string)
                .context("Failed to establish connection.")?;

            conn.run_pending_migrations(MIGRATIONS)
                .map_err(|error| anyhow::anyhow!(error))
                .context("Failed to run migrations.")?;

            Ok(())
        });

        handle
            .await
            .map_err(|error| InitializeError {
                reason: error.to_string(),
            })?
            .map_err(|error| InitializeError {
                reason: error.to_string(),
            })?;

        tracing::trace!("Successfully initialized.");
        Ok(())
    }
}

#[async_trait]
impl Push for PostgresPlugin {
    #[tracing::instrument(name = "PostgresPlugin push_alert", skip_all)]
    async fn push_alert(&self, alertmanager_push: &AlermanagerPush) -> Result<(), PushError> {
        tracing::trace!("Pushing.");
        let conn = self.pool.get().await.map_err(|error| PushError {
            reason: error.to_string(),
        })?;

        {
            let _conn = conn;
            let _alert = alertmanager_push;
        }

        tracing::trace!("Successfully pushed.");
        Ok(())
    }
}
