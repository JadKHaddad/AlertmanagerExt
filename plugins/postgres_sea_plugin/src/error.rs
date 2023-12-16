use push_definitions::{InitializeError, PushError};
use sea_orm::error::DbErr;
use thiserror::Error as ThisError;

#[derive(ThisError, Debug)]
pub enum InternalPushError {
    #[error("Error beginning transaction error: {0}")]
    TransactionBegin(#[source] DbErr),
    #[error("Error committing transaction error: {0}")]
    TransactionCommit(#[source] DbErr),
    #[error("Error inserting alert group. group_key: {group_key}, error: {error}")]
    GroupInsertion {
        group_key: String,
        #[source]
        error: DbErr,
    },
    #[error("Error getting group label id. group_key: {group_key}, label_name: {label_name}, label_value: {label_value}, error: {error}")]
    GroupLabelId {
        group_key: String,
        label_name: String,
        label_value: String,
        #[source]
        error: DbErr,
    },
    #[error("Error inserting group label. group_key: {group_key}, label_name: {label_name}, label_value: {label_value}, error: {error}")]
    GroupLabelInsertion {
        group_key: String,
        label_name: String,
        label_value: String,
        #[source]
        error: DbErr,
    },
    #[error("Error assigning group label. group_key: {group_key}, label_name: {label_name}, label_value: {label_value}, error: {error}")]
    GroupLabelAssignment {
        group_key: String,
        label_name: String,
        label_value: String,
        #[source]
        error: DbErr,
    },
    #[error("Error getting common label id. group_key: {group_key}, label_name: {label_name}, label_value: {label_value}, error: {error}")]
    CommonLabelId {
        group_key: String,
        label_name: String,
        label_value: String,
        #[source]
        error: DbErr,
    },
    #[error("Error inserting common label. group_key: {group_key}, label_name: {label_name}, label_value: {label_value}, error: {error}")]
    CommonLabelInsertion {
        group_key: String,
        label_name: String,
        label_value: String,
        #[source]
        error: DbErr,
    },
    #[error("Error assigning common label. group_key: {group_key}, label_name: {label_name}, label_value: {label_value}, error: {error}")]
    CommonLabelAssignment {
        group_key: String,
        label_name: String,
        label_value: String,
        #[source]
        error: DbErr,
    },
    #[error("Error getting common annotation id. group_key: {group_key}, annotation_name: {annotation_name}, annotation_value: {annotation_value}, error: {error}")]
    CommonAnnotationId {
        group_key: String,
        annotation_name: String,
        annotation_value: String,
        #[source]
        error: DbErr,
    },
    #[error("Error inserting common annotation. group_key: {group_key}, annotation_name: {annotation_name}, annotation_value: {annotation_value}, error: {error}")]
    CommonAnnotationInsertion {
        group_key: String,
        annotation_name: String,
        annotation_value: String,
        #[source]
        error: DbErr,
    },
    #[error("Error assigning common annotation. group_key: {group_key}, annotation_name: {annotation_name}, annotation_value: {annotation_value}, error: {error}")]
    CommonAnnotationAssignment {
        group_key: String,
        annotation_name: String,
        annotation_value: String,
        #[source]
        error: DbErr,
    },
    #[error(
        "Error inserting alert. group_key: {group_key}, fingerprint: {fingerprint}, error: {error}"
    )]
    AlertInsertion {
        group_key: String,
        fingerprint: String,
        #[source]
        error: DbErr,
    },
    #[error("Error getting alert label id. group_key: {group_key}, fingerprint: {fingerprint}, label_name: {label_name}, label_value: {label_value}, error: {error}")]
    AlertLabelId {
        group_key: String,
        fingerprint: String,
        label_name: String,
        label_value: String,
        #[source]
        error: DbErr,
    },
    #[error("Error inserting alert label. group_key: {group_key}, fingerprint: {fingerprint}, label_name: {label_name}, label_value: {label_value}, error: {error}")]
    AlertLabelInsertion {
        group_key: String,
        fingerprint: String,
        label_name: String,
        label_value: String,
        #[source]
        error: DbErr,
    },
    #[error("Error assigning alert label. group_key: {group_key}, fingerprint: {fingerprint}, label_name: {label_name}, label_value: {label_value}, error: {error}")]
    AlertLabelAssignment {
        group_key: String,
        fingerprint: String,
        label_name: String,
        label_value: String,
        #[source]
        error: DbErr,
    },
    #[error("Error getting alert annotation id. group_key: {group_key}, fingerprint: {fingerprint}, annotation_name: {annotation_name}, annotation_value: {annotation_value}, error: {error}")]
    AlertAnnotationId {
        group_key: String,
        fingerprint: String,
        annotation_name: String,
        annotation_value: String,
        #[source]
        error: DbErr,
    },
    #[error("Error inserting alert annotation. group_key: {group_key}, fingerprint: {fingerprint}, annotation_name: {annotation_name}, annotation_value: {annotation_value}, error: {error}")]
    AlertAnnotationInsertion {
        group_key: String,
        fingerprint: String,
        annotation_name: String,
        annotation_value: String,
        #[source]
        error: DbErr,
    },
    #[error("Error assigning alert annotation. group_key: {group_key}, fingerprint: {fingerprint}, annotation_name: {annotation_name}, annotation_value: {annotation_value}, error: {error}")]
    AlertAnnotationAssignment {
        group_key: String,
        fingerprint: String,
        annotation_name: String,
        annotation_value: String,
        #[source]
        error: DbErr,
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
        DbErr,
    ),
}

impl From<InternalInitializeError> for InitializeError {
    fn from(error: InternalInitializeError) -> Self {
        Self {
            error: error.into(),
        }
    }
}
