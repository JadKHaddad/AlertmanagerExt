// @generated automatically by Diesel CLI.

diesel::table! {
    alerts (id) {
        id -> Nullable<Integer>,
        group_id -> Integer,
        group_key -> Text,
        status -> Text,
        starts_at -> Timestamp,
        ends_at -> Nullable<Timestamp>,
        generator_url -> Text,
        fingerprint -> Text,
    }
}

diesel::table! {
    alerts_annotations (alert_id, annotation_id) {
        alert_id -> Integer,
        annotation_id -> Integer,
    }
}

diesel::table! {
    alerts_labels (alert_id, label_id) {
        alert_id -> Integer,
        label_id -> Integer,
    }
}

diesel::table! {
    annotations (id) {
        id -> Nullable<Integer>,
        name -> Text,
        value -> Text,
    }
}

diesel::table! {
    common_annotations (id) {
        id -> Nullable<Integer>,
        name -> Text,
        value -> Text,
    }
}

diesel::table! {
    common_labels (id) {
        id -> Nullable<Integer>,
        name -> Text,
        value -> Text,
    }
}

diesel::table! {
    groups (id) {
        id -> Nullable<Integer>,
        timestamp -> Timestamp,
        group_key -> Text,
        receiver -> Text,
        status -> Text,
        external_url -> Text,
    }
}

diesel::table! {
    groups_common_annotations (group_id, common_annotation_id) {
        group_id -> Integer,
        common_annotation_id -> Integer,
    }
}

diesel::table! {
    groups_common_labels (group_id, common_label_id) {
        group_id -> Integer,
        common_label_id -> Integer,
    }
}

diesel::table! {
    groups_labels (group_id, label_id) {
        group_id -> Integer,
        label_id -> Integer,
    }
}

diesel::table! {
    labels (id) {
        id -> Nullable<Integer>,
        name -> Text,
        value -> Text,
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
