CREATE TABLE
    alerts_annotations (
        alert_id INTEGER NOT NULL references alerts(id),
        annotation_id INTEGER NOT NULL references annotations(id),
        PRIMARY KEY (alert_id, annotation_id)
    );