CREATE TABLE
    groups_common_labels (
        group_id INTEGER NOT NULL references groups(id),
        common_label_id INTEGER NOT NULL references common_labels(id),
        PRIMARY KEY (group_id, common_label_id)
    );