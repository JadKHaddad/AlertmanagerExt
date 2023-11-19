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
    alert_annotation (id) {
        id -> Int4,
        alert_id -> Int4,
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
    alert_label (id) {
        id -> Int4,
        alert_id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        value -> Varchar,
    }
}

diesel::table! {
    common_annotation (id) {
        id -> Int4,
        alert_group_id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        value -> Varchar,
    }
}

diesel::table! {
    common_label (id) {
        id -> Int4,
        alert_group_id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        value -> Varchar,
    }
}

diesel::table! {
    group_label (id) {
        id -> Int4,
        alert_group_id -> Int4,
        #[max_length = 255]
        name -> Varchar,
        #[max_length = 255]
        value -> Varchar,
    }
}

diesel::joinable!(alert -> alert_group (alert_group_id));
diesel::joinable!(alert_annotation -> alert (alert_id));
diesel::joinable!(alert_label -> alert (alert_id));
diesel::joinable!(common_annotation -> alert_group (alert_group_id));
diesel::joinable!(common_label -> alert_group (alert_group_id));
diesel::joinable!(group_label -> alert_group (alert_group_id));

diesel::allow_tables_to_appear_in_same_query!(
    alert,
    alert_annotation,
    alert_group,
    alert_label,
    common_annotation,
    common_label,
    group_label,
);
