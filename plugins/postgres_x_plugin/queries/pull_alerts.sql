SELECT
    alerts.id,
    group_key,
    status AS "status: AlertStatusModel",
    starts_at,
    ends_at,
    generator_url,
    fingerprint,
    labels as "labels!: Vec<Label>",
    annotations as "annotations!: Vec<Annotation>"
FROM (
        SELECT
            labels_per_alert.id,
            labels_per_alert.labels,
            ARRAY_AGG( (annotations)) AS annotations
        FROM (
                SELECT
                    alerts.id,
                    ARRAY_AGG( (labels)) AS labels
                FROM alerts
                    INNER JOIN alerts_labels ON alerts_labels.alert_id = alerts.id
                    INNER JOIN labels ON labels.id = alerts_labels.label_id
                GROUP BY
                    alerts.id
            ) AS labels_per_alert
            INNER JOIN alerts_annotations ON alerts_annotations.alert_id = labels_per_alert.id
            INNER JOIN annotations ON annotations.id = alerts_annotations.annotation_id
        GROUP BY
            labels_per_alert.id,
            labels_per_alert.labels
    ) AS labels_and_annotations_per_alert
    INNER JOIN alerts ON alerts.id = labels_and_annotations_per_alert.id;