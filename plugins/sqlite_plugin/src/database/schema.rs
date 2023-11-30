// @generated automatically by Diesel CLI.

diesel::table! {
    alert (id) {
        id -> Nullable<Integer>,
        alert_group_id -> Integer,
        status -> Text,
        starts_at -> Timestamp,
        ends_at -> Nullable<Timestamp>,
        generator_url -> Text,
        fingerprint -> Text,
    }
}

diesel::table! {
    alert_annotation (id) {
        id -> Nullable<Integer>,
        alert_id -> Integer,
        name -> Text,
        value -> Text,
    }
}

diesel::table! {
    alert_group (id) {
        id -> Nullable<Integer>,
        timestamp -> Timestamp,
        group_key -> Text,
        receiver -> Text,
        status -> Text,
        external_url -> Text,
    }
}

diesel::table! {
    alert_label (id) {
        id -> Nullable<Integer>,
        alert_id -> Integer,
        name -> Text,
        value -> Text,
    }
}

diesel::table! {
    common_annotation (id) {
        id -> Nullable<Integer>,
        alert_group_id -> Integer,
        name -> Text,
        value -> Text,
    }
}

diesel::table! {
    common_label (id) {
        id -> Nullable<Integer>,
        alert_group_id -> Integer,
        name -> Text,
        value -> Text,
    }
}

diesel::table! {
    group_label (id) {
        id -> Nullable<Integer>,
        alert_group_id -> Integer,
        name -> Text,
        value -> Text,
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
