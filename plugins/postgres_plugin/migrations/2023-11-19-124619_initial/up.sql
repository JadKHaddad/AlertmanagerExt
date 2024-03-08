-- Your SQL goes here

CREATE TYPE alert_status AS ENUM ('resolved', 'firing');

CREATE TABLE
    groups(
        id SERIAL PRIMARY KEY,
        timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
        group_key VARCHAR(255) NOT NULL,
        receiver VARCHAR(255) NOT NULL,
        status alert_status NOT NULL,
        external_url VARCHAR(255) NOT NULL,
        UNIQUE (group_key)
    );

CREATE TABLE
    alerts (
        id SERIAL PRIMARY KEY,
        group_id INTEGER NOT NULL references groups(id) ON DELETE CASCADE ON UPDATE CASCADE,
        group_key VARCHAR(255) NOT NULL references groups(group_key) ON DELETE CASCADE ON UPDATE CASCADE,
        status alert_status NOT NULL,
        starts_at TIMESTAMP NOT NULL,
        ends_at TIMESTAMP,
        generator_url VARCHAR(255) NOT NULL,
        fingerprint VARCHAR(255) NOT NULL,
        UNIQUE (group_key, fingerprint)
    );

CREATE TABLE
    labels (
        id SERIAL PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
        value VARCHAR(255) NOT NULL,
        UNIQUE (name, value)
    );

CREATE TABLE
    annotations (
        id SERIAL PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
        value VARCHAR(255) NOT NULL,
        UNIQUE (name, value)
    );

CREATE TABLE
    common_labels (
        id SERIAL PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
        value VARCHAR(255) NOT NULL,
        UNIQUE (name, value)
    );

CREATE TABLE
    common_annotations (
        id SERIAL PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
        value VARCHAR(255) NOT NULL,
        UNIQUE (name, value)
    );

CREATE TABLE
    groups_labels (
        group_id INTEGER NOT NULL references groups(id) ON DELETE CASCADE ON UPDATE CASCADE,
        label_id INTEGER NOT NULL references labels(id) ON DELETE CASCADE ON UPDATE CASCADE,
        PRIMARY KEY (group_id, label_id)
    );

CREATE TABLE
    groups_common_labels (
        group_id INTEGER NOT NULL references groups(id) ON DELETE CASCADE ON UPDATE CASCADE,
        common_label_id INTEGER NOT NULL references common_labels(id) ON DELETE CASCADE ON UPDATE CASCADE,
        PRIMARY KEY (group_id, common_label_id)
    );

CREATE TABLE
    groups_common_annotations (
        group_id INTEGER NOT NULL references groups(id) ON DELETE CASCADE ON UPDATE CASCADE,
        common_annotation_id INTEGER NOT NULL references common_annotations(id) ON DELETE CASCADE ON UPDATE CASCADE,
        PRIMARY KEY (
            group_id,
            common_annotation_id
        )
    );

CREATE TABLE
    alerts_labels (
        alert_id INTEGER NOT NULL references alerts(id) ON DELETE CASCADE ON UPDATE CASCADE,
        label_id INTEGER NOT NULL references labels(id) ON DELETE CASCADE ON UPDATE CASCADE,
        PRIMARY KEY (alert_id, label_id)
    );

CREATE TABLE
    alerts_annotations (
        alert_id INTEGER NOT NULL references alerts(id) ON DELETE CASCADE ON UPDATE CASCADE,
        annotation_id INTEGER NOT NULL references annotations(id) ON DELETE CASCADE ON UPDATE CASCADE,
        PRIMARY KEY (alert_id, annotation_id)
    );