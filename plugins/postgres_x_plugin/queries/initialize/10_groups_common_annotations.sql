CREATE TABLE
    groups_common_annotations (
        group_id INTEGER NOT NULL references groups(id),
        common_annotation_id INTEGER NOT NULL references common_annotations(id),
        PRIMARY KEY (
            group_id,
            common_annotation_id
        )
    );