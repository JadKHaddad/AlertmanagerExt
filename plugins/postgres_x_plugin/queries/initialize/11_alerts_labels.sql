CREATE TABLE
    alerts_labels (
        alert_id INTEGER NOT NULL references alerts(id),
        label_id INTEGER NOT NULL references labels(id),
        PRIMARY KEY (alert_id, label_id)
    );