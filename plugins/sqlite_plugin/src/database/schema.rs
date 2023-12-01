// @generated automatically by Diesel CLI.

diesel::table! {
    alert (id) {
        id -> Nullable<Integer>,
        alert_group_id -> Integer,
        group_key -> Text,
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
        name -> Text,
        value -> Text,
    }
}

diesel::table! {
    assign_alert_annotation (id) {
        id -> Nullable<Integer>,
        alert_id -> Integer,
        alert_annotation_id -> Integer,
    }
}

diesel::table! {
    assign_alert_label (id) {
        id -> Nullable<Integer>,
        alert_id -> Integer,
        alert_label_id -> Integer,
    }
}

diesel::table! {
    assign_common_annotation (id) {
        id -> Nullable<Integer>,
        alert_group_id -> Integer,
        common_annotation_id -> Integer,
    }
}

diesel::table! {
    assign_common_label (id) {
        id -> Nullable<Integer>,
        alert_group_id -> Integer,
        common_label_id -> Integer,
    }
}

diesel::table! {
    assign_group_label (id) {
        id -> Nullable<Integer>,
        alert_group_id -> Integer,
        group_label_id -> Integer,
    }
}

diesel::table! {
    common_annotation (id) {
        id -> Nullable<Integer>,
        name -> Text,
        value -> Text,
    }
}

diesel::table! {
    common_label (id) {
        id -> Nullable<Integer>,
        name -> Text,
        value -> Text,
    }
}

diesel::table! {
    group_label (id) {
        id -> Nullable<Integer>,
        name -> Text,
        value -> Text,
    }
}

diesel::joinable!(assign_alert_annotation -> alert (alert_id));
diesel::joinable!(assign_alert_annotation -> alert_annotation (alert_annotation_id));
diesel::joinable!(assign_alert_label -> alert (alert_id));
diesel::joinable!(assign_alert_label -> alert_label (alert_label_id));
diesel::joinable!(assign_common_annotation -> alert_group (alert_group_id));
diesel::joinable!(assign_common_annotation -> common_annotation (common_annotation_id));
diesel::joinable!(assign_common_label -> alert_group (alert_group_id));
diesel::joinable!(assign_common_label -> common_label (common_label_id));
diesel::joinable!(assign_group_label -> alert_group (alert_group_id));
diesel::joinable!(assign_group_label -> group_label (group_label_id));

diesel::allow_tables_to_appear_in_same_query!(
    alert,
    alert_annotation,
    alert_group,
    alert_label,
    assign_alert_annotation,
    assign_alert_label,
    assign_common_annotation,
    assign_common_label,
    assign_group_label,
    common_annotation,
    common_label,
    group_label,
);
