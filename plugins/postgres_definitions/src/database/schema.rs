// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "alert_status"))]
    pub struct AlertStatus;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::AlertStatus;

    alerts (id) {
        id -> Int4,
        group_id -> Int4,
        #[max_length = 255]
        group_key -> Varchar,
        status -> AlertStatus,
        starts_at -> Timestamp,
        ends_at -> Nullable<Timestamp>,
        #[max_length = 255]
        generator_url -> Varchar,
        #[max_length = 255]
        fingerprint -> Varchar,
    }
}

diesel::table! {
    alerts_annotations (alert_id, annotation_id) {
        alert_id -> Int4,
        annotation_id -> Int4,
    }
}

diesel::table! {
    alerts_labels (alert_id, label_id) {
        alert_id -> Int4,
        label_id -> Int4,
    }
}

diesel::table! {
    annotations (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        value -> Varchar,
    }
}

diesel::table! {
    common_annotations (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        value -> Varchar,
    }
}

diesel::table! {
    common_labels (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        value -> Varchar,
    }
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::AlertStatus;

    groups (id) {
        id -> Int4,
        timestamp -> Timestamp,
        #[max_length = 255]
        group_key -> Varchar,
        #[max_length = 255]
        receiver -> Varchar,
        status -> AlertStatus,
        #[max_length = 255]
        external_url -> Varchar,
    }
}

diesel::table! {
    groups_common_annotations (group_id, common_annotation_id) {
        group_id -> Int4,
        common_annotation_id -> Int4,
    }
}

diesel::table! {
    groups_common_labels (group_id, common_label_id) {
        group_id -> Int4,
        common_label_id -> Int4,
    }
}

diesel::table! {
    groups_labels (group_id, label_id) {
        group_id -> Int4,
        label_id -> Int4,
    }
}

diesel::table! {
    labels (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        value -> Varchar,
    }
}

diesel::joinable!(alerts_annotations -> alerts (alert_id));
diesel::joinable!(alerts_annotations -> annotations (annotation_id));
diesel::joinable!(alerts_labels -> alerts (alert_id));
diesel::joinable!(alerts_labels -> labels (label_id));
diesel::joinable!(groups_common_annotations -> common_annotations (common_annotation_id));
diesel::joinable!(groups_common_annotations -> groups (group_id));
diesel::joinable!(groups_common_labels -> common_labels (common_label_id));
diesel::joinable!(groups_common_labels -> groups (group_id));
diesel::joinable!(groups_labels -> groups (group_id));
diesel::joinable!(groups_labels -> labels (label_id));

diesel::allow_tables_to_appear_in_same_query!(
    alerts,
    alerts_annotations,
    alerts_labels,
    annotations,
    common_annotations,
    common_labels,
    groups,
    groups_common_annotations,
    groups_common_labels,
    groups_labels,
    labels,
);
