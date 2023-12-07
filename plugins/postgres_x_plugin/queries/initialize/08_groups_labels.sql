CREATE TABLE
    groups_labels (
        group_id INTEGER NOT NULL references groups(id),
        label_id INTEGER NOT NULL references labels(id),
        PRIMARY KEY (group_id, label_id)
    );