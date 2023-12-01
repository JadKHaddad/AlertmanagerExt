// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "alert_status"))]
    pub struct AlertStatus;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::AlertStatus;

    alert (id) {
        id -> Int4,
        alert_group_id -> Int4,
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
    alert_alert_annotations (id) {
        id -> Int4,
        alert_id -> Int4,
        alert_annotation_id -> Int4,
    }
}

diesel::table! {
    alert_alert_labels (id) {
        id -> Int4,
        alert_id -> Int4,
        alert_label_id -> Int4,
    }
}

diesel::table! {
    alert_annotation (id) {
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

    alert_group (id) {
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
    alert_group_common_annotations (id) {
        id -> Int4,
        alert_group_id -> Int4,
        common_annotation_id -> Int4,
    }
}

diesel::table! {
    alert_group_common_labels (id) {
        id -> Int4,
        alert_group_id -> Int4,
        common_label_id -> Int4,
    }
}

diesel::table! {
    alert_group_group_labels (id) {
        id -> Int4,
        alert_group_id -> Int4,
        group_label_id -> Int4,
    }
}

diesel::table! {
    alert_label (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        value -> Varchar,
    }
}

diesel::table! {
    common_annotation (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        value -> Varchar,
    }
}

diesel::table! {
    common_label (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        value -> Varchar,
    }
}

diesel::table! {
    group_label (id) {
        id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        value -> Varchar,
    }
}

diesel::joinable!(alert_alert_annotations -> alert (alert_id));
diesel::joinable!(alert_alert_annotations -> alert_annotation (alert_annotation_id));
diesel::joinable!(alert_alert_labels -> alert (alert_id));
diesel::joinable!(alert_alert_labels -> alert_label (alert_label_id));
diesel::joinable!(alert_group_common_annotations -> alert_group (alert_group_id));
diesel::joinable!(alert_group_common_annotations -> common_annotation (common_annotation_id));
diesel::joinable!(alert_group_common_labels -> alert_group (alert_group_id));
diesel::joinable!(alert_group_common_labels -> common_label (common_label_id));
diesel::joinable!(alert_group_group_labels -> alert_group (alert_group_id));
diesel::joinable!(alert_group_group_labels -> group_label (group_label_id));

diesel::allow_tables_to_appear_in_same_query!(
    alert,
    alert_alert_annotations,
    alert_alert_labels,
    alert_annotation,
    alert_group,
    alert_group_common_annotations,
    alert_group_common_labels,
    alert_group_group_labels,
    alert_label,
    common_annotation,
    common_label,
    group_label,
);
