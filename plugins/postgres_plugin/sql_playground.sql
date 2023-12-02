select *
from (
        select
            grouped_with_labels.id,
            grouped_with_labels.labels,
            array_agg( (
                    alert_annotation.name,
                    alert_annotation.value
                )
            ) as annotations
        from (
                select
                    alert.id,
                    array_agg( (
                            alert_label.name,
                            alert_label.value
                        )
                    ) as labels
                from alert
                    inner join assign_alert_label ON assign_alert_label.alert_id = alert.id
                    inner join alert_label ON alert_label.id = assign_alert_label.alert_label_id
                group by
                    alert.id,
                    alert.fingerprint
            ) as grouped_with_labels
            inner join assign_alert_annotation ON assign_alert_annotation.alert_id = grouped_with_labels.id
            inner join alert_annotation ON alert_annotation.id = assign_alert_annotation.alert_annotation_id
        group by
            grouped_with_labels.id,
            grouped_with_labels.labels
    ) as grouped_with_lalbels_and_annotations
    inner join alert on alert.id = grouped_with_lalbels_and_annotations.id;