use push_definitions::{InitializeError, PushError};
use sqlx::Error as SqlxError;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum InternalInitializeError {
    #[error("Failed to create alert_status type: {0}")]
    AlertStatus(#[source] SqlxError),
    #[error("Failed to create groups table: {0}")]
    Groups(#[source] SqlxError),
    #[error("Failed to create alerts table: {0}")]
    Alerts(#[source] SqlxError),
    #[error("Failed to create labels table: {0}")]
    Labels(#[source] SqlxError),
    #[error("Failed to create annotations table: {0}")]
    Annotations(#[source] SqlxError),
    #[error("Failed to create common_labels table: {0}")]
    CommonLabels(#[source] SqlxError),
    #[error("Failed to create common_annotations table: {0}")]
    CommonAnnotations(#[source] SqlxError),
    #[error("Failed to create groups_labels table: {0}")]
    GroupsLabels(#[source] SqlxError),
    #[error("Failed to create groups_common_labels table: {0}")]
    GroupsCommonLabels(#[source] SqlxError),
    #[error("Failed to create groups_common_annotations table: {0}")]
    GroupsCommonAnnotations(#[source] SqlxError),
    #[error("Failed to create alerts_labels table: {0}")]
    AlertsLabels(#[source] SqlxError),
    #[error("Failed to create alerts_annotations table: {0}")]
    AlertsAnnotations(#[source] SqlxError),
}

#[derive(ThisError, Debug)]
pub enum InternalPushError {
    #[error("Error getting connection from pool: {0}")]
    Acquire(#[source] SqlxError),
    #[error("Error starting transaction error: {0}")]
    TransactionStart(#[source] SqlxError),
    #[error("Error committing transaction error: {0}")]
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

impl From<InternalInitializeError> for InitializeError {
    fn from(error: InternalInitializeError) -> Self {
        Self {
            reason: error.to_string(),
        }
    }
}

impl From<InternalPushError> for PushError {
    fn from(error: InternalPushError) -> Self {
        Self {
            reason: error.to_string(),
        }
    }
}
