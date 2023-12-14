use bb8::RunError;
use diesel::{result::Error as DieselError, ConnectionError};
use diesel_async::pooled_connection::PoolError;
use plugins_definitions::HealthError;
use pull_definitions::PullError;
use push_definitions::{InitializeError, PushError};
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum NewPostgresPluginError {
    #[error("Failed to create pool: {0}")]
    Pool(
        #[source]
        #[from]
        PoolError,
    ),
}

#[derive(ThisError, Debug)]
pub enum InternalPushError {
    #[error("Error getting connection from pool: {0}")]
    Acquire(#[source] RunError<PoolError>),
    #[error("Transaction error: {0}")]
    Transaction(
        #[source]
        #[from]
        DieselError,
    ),
    #[error("Error inserting alert group. group_key: {group_key}, error: {error}")]
    GroupInsertion {
        group_key: String,
        #[source]
        error: DieselError,
    },
    #[error("Error getting group label id. group_key: {group_key}, label_name: {label_name}, label_value: {label_value}, error: {error}")]
    GroupLabelId {
        group_key: String,
        label_name: String,
        label_value: String,
        #[source]
        error: DieselError,
    },
    #[error("Error inserting group label. group_key: {group_key}, label_name: {label_name}, label_value: {label_value}, error: {error}")]
    GroupLabelInsertion {
        group_key: String,
        label_name: String,
        label_value: String,
        #[source]
        error: DieselError,
    },
    #[error("Error assigning group label. group_key: {group_key}, label_name: {label_name}, label_value: {label_value}, error: {error}")]
    GroupLabelAssignment {
        group_key: String,
        label_name: String,
        label_value: String,
        #[source]
        error: DieselError,
    },
    #[error("Error getting common label id. group_key: {group_key}, label_name: {label_name}, label_value: {label_value}, error: {error}")]
    CommonLabelId {
        group_key: String,
        label_name: String,
        label_value: String,
        #[source]
        error: DieselError,
    },
    #[error("Error inserting common label. group_key: {group_key}, label_name: {label_name}, label_value: {label_value}, error: {error}")]
    CommonLabelInsertion {
        group_key: String,
        label_name: String,
        label_value: String,
        #[source]
        error: DieselError,
    },
    #[error("Error assigning common label. group_key: {group_key}, label_name: {label_name}, label_value: {label_value}, error: {error}")]
    CommonLabelAssignment {
        group_key: String,
        label_name: String,
        label_value: String,
        #[source]
        error: DieselError,
    },
    #[error("Error getting common annotation id. group_key: {group_key}, annotation_name: {annotation_name}, annotation_value: {annotation_value}, error: {error}")]
    CommonAnnotationId {
        group_key: String,
        annotation_name: String,
        annotation_value: String,
        #[source]
        error: DieselError,
    },
    #[error("Error inserting common annotation. group_key: {group_key}, annotation_name: {annotation_name}, annotation_value: {annotation_value}, error: {error}")]
    CommonAnnotationInsertion {
        group_key: String,
        annotation_name: String,
        annotation_value: String,
        #[source]
        error: DieselError,
    },
    #[error("Error assigning common annotation. group_key: {group_key}, annotation_name: {annotation_name}, annotation_value: {annotation_value}, error: {error}")]
    CommonAnnotationAssignment {
        group_key: String,
        annotation_name: String,
        annotation_value: String,
        #[source]
        error: DieselError,
    },
    #[error(
        "Error inserting alert. group_key: {group_key}, fingerprint: {fingerprint}, error: {error}"
    )]
    AlertInsertion {
        group_key: String,
        fingerprint: String,
        #[source]
        error: DieselError,
    },
    #[error("Error getting alert label id. group_key: {group_key}, fingerprint: {fingerprint}, label_name: {label_name}, label_value: {label_value}, error: {error}")]
    AlertLabelId {
        group_key: String,
        fingerprint: String,
        label_name: String,
        label_value: String,
        #[source]
        error: DieselError,
    },
    #[error("Error inserting alert label. group_key: {group_key}, fingerprint: {fingerprint}, label_name: {label_name}, label_value: {label_value}, error: {error}")]
    AlertLabelInsertion {
        group_key: String,
        fingerprint: String,
        label_name: String,
        label_value: String,
        #[source]
        error: DieselError,
    },
    #[error("Error assigning alert label. group_key: {group_key}, fingerprint: {fingerprint}, label_name: {label_name}, label_value: {label_value}, error: {error}")]
    AlertLabelAssignment {
        group_key: String,
        fingerprint: String,
        label_name: String,
        label_value: String,
        #[source]
        error: DieselError,
    },
    #[error("Error getting alert annotation id. group_key: {group_key}, fingerprint: {fingerprint}, annotation_name: {annotation_name}, annotation_value: {annotation_value}, error: {error}")]
    AlertAnnotationId {
        group_key: String,
        fingerprint: String,
        annotation_name: String,
        annotation_value: String,
        #[source]
        error: DieselError,
    },
    #[error("Error inserting alert annotation. group_key: {group_key}, fingerprint: {fingerprint}, annotation_name: {annotation_name}, annotation_value: {annotation_value}, error: {error}")]
    AlertAnnotationInsertion {
        group_key: String,
        fingerprint: String,
        annotation_name: String,
        annotation_value: String,
        #[source]
        error: DieselError,
    },
    #[error("Error assigning alert annotation. group_key: {group_key}, fingerprint: {fingerprint}, annotation_name: {annotation_name}, annotation_value: {annotation_value}, error: {error}")]
    AlertAnnotationAssignment {
        group_key: String,
        fingerprint: String,
        annotation_name: String,
        annotation_value: String,
        #[source]
        error: DieselError,
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
/// Error inserting a label
///
/// Only labels are shared between [`crate::database::models::groups::Group`] and [`crate::database::models::alerts::Alert`].
/// So this error is used for both.
pub enum LablelInsertionError {
    #[error("Get error: {0}")]
    Get(#[source] DieselError),
    #[error("Insert error: {0}")]
    Insert(#[source] DieselError),
}

#[derive(ThisError, Debug)]
pub enum InternalPullError {
    #[error("Error getting connection from pool: {0}")]
    Acquire(#[source] RunError<PoolError>),
    #[error("Error getting alerts: {0}")]
    Alerts(#[source] DieselError),
    #[error("Error getting labels: {0}")]
    Labels(#[source] DieselError),
    #[error("Error getting annotations: {0}")]
    Annotations(#[source] DieselError),
}

impl From<InternalPullError> for PullError {
    fn from(error: InternalPullError) -> Self {
        Self {
            error: error.into(),
        }
    }
}

#[derive(ThisError, Debug)]
pub enum InternalInitializeError {
    #[error("Already initialized")]
    AlreadyInitialized,
    #[error("Failed to establish connection: {0}")]
    Connection(
        #[from]
        #[source]
        ConnectionError,
    ),
    #[error("Failed to run migrations: {0}")]
    Migrations(#[source] Box<dyn std::error::Error + Send + Sync>),
    #[error("Failed to join task: {0}")]
    Join(
        #[from]
        #[source]
        tokio::task::JoinError,
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
    #[error("Failed to get connection: {0}")]
    Connection(
        #[from]
        #[source]
        bb8::RunError<PoolError>,
    ),
}

impl From<InternalHealthError> for HealthError {
    fn from(error: InternalHealthError) -> Self {
        Self {
            error: error.into(),
        }
    }
}
