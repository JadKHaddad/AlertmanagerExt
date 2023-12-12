use mongodb::error::Error as MongoError;
use plugins_definitions::HealthError;
use push_definitions::PushError;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum NewMongoPluginError {
    #[error("Failed to parse connection string: {0}")]
    Parse(#[source] MongoError),
    #[error("Failed to create client: {0}")]
    Client(#[source] MongoError),
}

#[derive(ThisError, Debug)]
pub enum InternalHealthError {
    #[error("Failed to ping mongo: {0}")]
    Ping(
        #[source]
        #[from]
        MongoError,
    ),
}

impl From<InternalHealthError> for HealthError {
    fn from(error: InternalHealthError) -> Self {
        Self {
            error: error.into(),
        }
    }
}

#[derive(ThisError, Debug)]
pub enum InternalPushError {
    #[error("Error starting session: {error}")]
    StartSession {
        #[source]
        error: MongoError,
    },
    #[error("Error beginning transaction: {error}")]
    TransactionBegin {
        #[source]
        error: MongoError,
    },
    #[error("Error committing transaction: {error}")]
    CommitTransaction {
        #[source]
        error: MongoError,
    },
    #[error("Error inserting alert group: group_key: {group_key}, error: {error}")]
    GroupInsertion {
        group_key: String,
        #[source]
        error: MongoError,
    },
    #[error("Error obtaining alert group id: group_key: {group_key}")]
    GroupId { group_key: String },
    #[error("Error inserting group labels: group_key: {group_key}, error: {error}")]
    GroupLabelsInsertion {
        group_key: String,
        #[source]
        error: MongoError,
    },
    #[error("Error inserting common labels: group_key: {group_key}, error: {error}")]
    CommonLabelsInsertion {
        group_key: String,
        #[source]
        error: MongoError,
    },
    #[error("Error inserting common annotations: group_key: {group_key}, error: {error}")]
    CommonAnnotationsInsertion {
        group_key: String,
        #[source]
        error: MongoError,
    },
    #[error(
        "Error inserting alert: group_key: {group_key}, fingerprint: {fingerprint}, error: {error}"
    )]
    AlertInsertion {
        group_key: String,
        fingerprint: String,
        #[source]
        error: MongoError,
    },
    #[error("Error obtaining alertid: group_key: {group_key}, fingerprint: {fingerprint}")]
    AlertId {
        group_key: String,
        fingerprint: String,
    },
    #[error("Error inserting alert labels: group_key: {group_key}, fingerprint: {fingerprint}, error: {error}")]
    AlertLabelsInsertion {
        group_key: String,
        fingerprint: String,
        #[source]
        error: MongoError,
    },
    #[error("Error inserting alert annotations: group_key: {group_key}, fingerprint: {fingerprint}, error: {error}")]
    AlertAnnotationsInsertion {
        group_key: String,
        fingerprint: String,
        #[source]
        error: MongoError,
    },
}

impl From<InternalPushError> for PushError {
    fn from(error: InternalPushError) -> Self {
        PushError {
            error: error.into(),
        }
    }
}
