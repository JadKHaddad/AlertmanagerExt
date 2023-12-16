use plugins_definitions::HealthError;
use pull_definitions::PullError;
use push_definitions::{InitializeError, PushError};
use sqlx::{migrate::MigrateError, Error as SqlxError};
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum InternalPushError {
    #[error("Error getting connection from pool: {0}")]
    Acquire(#[source] SqlxError),
    #[error("Error beginning transaction: {0}")]
    TransactionBegin(#[source] SqlxError),
    #[error("Error committing transaction: {0}")]
    TransactionCommit(#[source] SqlxError),
    #[error("Error inserting alert group. group_key: {group_key}, error: {error}")]
    GroupInsertion {
        group_key: String,
        #[source]
        error: SqlxError,
    },
    #[error("Error getting group label id. group_key: {group_key}, label_name: {label_name}, label_value: {label_value}, error: {error}")]
    GroupLabelId {
        group_key: String,
        label_name: String,
        label_value: String,
        #[source]
        error: SqlxError,
    },
    #[error("Error inserting group label. group_key: {group_key}, label_name: {label_name}, label_value: {label_value}, error: {error}")]
    GroupLabelInsertion {
        group_key: String,
        label_name: String,
        label_value: String,
        #[source]
        error: SqlxError,
    },
    #[error("Error assigning group label. group_key: {group_key}, label_name: {label_name}, label_value: {label_value}, error: {error}")]
    GroupLabelAssignment {
        group_key: String,
        label_name: String,
        label_value: String,
        #[source]
        error: SqlxError,
    },
    #[error("Error getting common label id. group_key: {group_key}, label_name: {label_name}, label_value: {label_value}, error: {error}")]
    CommonLabelId {
        group_key: String,
        label_name: String,
        label_value: String,
        #[source]
        error: SqlxError,
    },
    #[error("Error inserting common label. group_key: {group_key}, label_name: {label_name}, label_value: {label_value}, error: {error}")]
    CommonLabelInsertion {
        group_key: String,
        label_name: String,
        label_value: String,
        #[source]
        error: SqlxError,
    },
    #[error("Error assigning common label. group_key: {group_key}, label_name: {label_name}, label_value: {label_value}, error: {error}")]
    CommonLabelAssignment {
        group_key: String,
        label_name: String,
        label_value: String,
        #[source]
        error: SqlxError,
    },
    #[error("Error getting common annotation id. group_key: {group_key}, annotation_name: {annotation_name}, annotation_value: {annotation_value}, error: {error}")]
    CommonAnnotationId {
        group_key: String,
        annotation_name: String,
        annotation_value: String,
        #[source]
        error: SqlxError,
    },
    #[error("Error inserting common annotation. group_key: {group_key}, annotation_name: {annotation_name}, annotation_value: {annotation_value}, error: {error}")]
    CommonAnnotationInsertion {
        group_key: String,
        annotation_name: String,
        annotation_value: String,
        #[source]
        error: SqlxError,
    },
    #[error("Error assigning common annotation. group_key: {group_key}, annotation_name: {annotation_name}, annotation_value: {annotation_value}, error: {error}")]
    CommonAnnotationAssignment {
        group_key: String,
        annotation_name: String,
        annotation_value: String,
        #[source]
        error: SqlxError,
    },
    #[error(
        "Error inserting alert. group_key: {group_key}, fingerprint: {fingerprint}, error: {error}"
    )]
    AlertInsertion {
        group_key: String,
        fingerprint: String,
        #[source]
        error: SqlxError,
    },
    #[error("Error getting alert label id. group_key: {group_key}, fingerprint: {fingerprint}, label_name: {label_name}, label_value: {label_value}, error: {error}")]
    AlertLabelId {
        group_key: String,
        fingerprint: String,
        label_name: String,
        label_value: String,
        #[source]
        error: SqlxError,
    },
    #[error("Error inserting alert label. group_key: {group_key}, fingerprint: {fingerprint}, label_name: {label_name}, label_value: {label_value}, error: {error}")]
    AlertLabelInsertion {
        group_key: String,
        fingerprint: String,
        label_name: String,
        label_value: String,
        #[source]
        error: SqlxError,
    },
    #[error("Error assigning alert label. group_key: {group_key}, fingerprint: {fingerprint}, label_name: {label_name}, label_value: {label_value}, error: {error}")]
    AlertLabelAssignment {
        group_key: String,
        fingerprint: String,
        label_name: String,
        label_value: String,
        #[source]
        error: SqlxError,
    },
    #[error("Error getting alert annotation id. group_key: {group_key}, fingerprint: {fingerprint}, annotation_name: {annotation_name}, annotation_value: {annotation_value}, error: {error}")]
    AlertAnnotationId {
        group_key: String,
        fingerprint: String,
        annotation_name: String,
        annotation_value: String,
        #[source]
        error: SqlxError,
    },
    #[error("Error inserting alert annotation. group_key: {group_key}, fingerprint: {fingerprint}, annotation_name: {annotation_name}, annotation_value: {annotation_value}, error: {error}")]
    AlertAnnotationInsertion {
        group_key: String,
        fingerprint: String,
        annotation_name: String,
        annotation_value: String,
        #[source]
        error: SqlxError,
    },
    #[error("Error assigning alert annotation. group_key: {group_key}, fingerprint: {fingerprint}, annotation_name: {annotation_name}, annotation_value: {annotation_value}, error: {error}")]
    AlertAnnotationAssignment {
        group_key: String,
        fingerprint: String,
        annotation_name: String,
        annotation_value: String,
        #[source]
        error: SqlxError,
    },
}

impl From<InternalPushError> for PushError {
    fn from(error: InternalPushError) -> Self {
        Self {
            error: error.into(),
        }
    }
}

#[derive(ThisError, Debug)]
pub enum InternalInitializeError {
    #[error("Failed to run migrations: {0}")]
    Migrations(
        #[from]
        #[source]
        MigrateError,
    ),
}

impl From<InternalInitializeError> for InitializeError {
    fn from(error: InternalInitializeError) -> Self {
        Self {
            error: error.into(),
        }
    }
}

#[derive(ThisError, Debug)]
pub enum InternalHealthError {
    #[error("Error getting connection from pool: {0}")]
    Acquire(#[source] SqlxError),
    #[error("Failed to ping database: {0}")]
    Ping(#[source] SqlxError),
}

impl From<InternalHealthError> for HealthError {
    fn from(error: InternalHealthError) -> Self {
        Self {
            error: error.into(),
        }
    }
}

#[derive(ThisError, Debug)]
pub enum InternalPullError {
    #[error("Error performing query: {0}")]
    Query(
        #[from]
        #[source]
        SqlxError,
    ),
}

impl From<InternalPullError> for PullError {
    fn from(error: InternalPullError) -> Self {
        Self {
            error: error.into(),
        }
    }
}
