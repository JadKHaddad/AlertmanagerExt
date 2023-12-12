use mongodb::error::Error as MongoError;
use push_definitions::PushError;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum InternalPushError {
    #[error("Error while starting session, error: {error}")]
    StartSession {
        #[source]
        error: MongoError,
    },
    #[error("Error while starting transaction, error: {error}")]
    StartTransaction {
        #[source]
        error: MongoError,
    },
    #[error("Error while committing transaction, error: {error}")]
    CommitTransaction {
        #[source]
        error: MongoError,
    },
    #[error("Error while inserting alert group: group_key: {group_key}, error: {error}")]
    GroupInsertion {
        group_key: String,
        #[source]
        error: MongoError,
    },
    #[error("Error while obtaining alert group id: group_key: {group_key}")]
    GroupId { group_key: String },
    #[error("Error while inserting group labels: group_key: {group_key}, error: {error}")]
    GroupLabelsInsertion {
        group_key: String,
        #[source]
        error: MongoError,
    },
    #[error("Error while inserting common labels: group_key: {group_key}, error: {error}")]
    CommonLabelsInsertion {
        group_key: String,
        #[source]
        error: MongoError,
    },
    #[error("Error while inserting common annotations: group_key: {group_key}, error: {error}")]
    CommonAnnotationsInsertion {
        group_key: String,
        #[source]
        error: MongoError,
    },
    #[error("Error while inserting alert: group_key: {group_key}, fingerprint: {fingerprint}, error: {error}")]
    AlertInsertion {
        group_key: String,
        fingerprint: String,
        #[source]
        error: MongoError,
    },
    #[error("Error while obtaining alertid: group_key: {group_key}, fingerprint: {fingerprint}")]
    AlertId {
        group_key: String,
        fingerprint: String,
    },
    #[error("Error while inserting alert labels: group_key: {group_key}, fingerprint: {fingerprint}, error: {error}")]
    AlertLabelsInsertion {
        group_key: String,
        fingerprint: String,
        #[source]
        error: MongoError,
    },
    #[error("Error while inserting alert annotations: group_key: {group_key}, fingerprint: {fingerprint}, error: {error}")]
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
