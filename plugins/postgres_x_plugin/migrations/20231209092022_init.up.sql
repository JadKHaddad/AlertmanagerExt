-- Add up migration script here

CREATE TYPE alert_status AS ENUM ('resolved', 'firing');

CREATE TABLE
    IF NOT EXISTS groups(
        id SERIAL PRIMARY KEY,
        timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
        group_key VARCHAR(255) NOT NULL,
        receiver VARCHAR(255) NOT NULL,
        status alert_status NOT NULL,
        external_url VARCHAR(255) NOT NULL,
        UNIQUE (group_key)
    );

CREATE TABLE
    IF NOT EXISTS alerts (
        id SERIAL PRIMARY KEY,
        group_id INTEGER NOT NULL references groups(id),
        group_key VARCHAR(255) NOT NULL references groups(group_key),
        status alert_status NOT NULL,
        starts_at TIMESTAMP NOT NULL,
        ends_at TIMESTAMP,
        generator_url VARCHAR(255) NOT NULL,
        fingerprint VARCHAR(255) NOT NULL,
        UNIQUE (group_key, fingerprint)
    );

CREATE TABLE
    IF NOT EXISTS labels (
        id SERIAL PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
        value VARCHAR(255) NOT NULL,
        UNIQUE (name, value)
    );

CREATE TABLE
    IF NOT EXISTS annotations (
        id SERIAL PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
        value VARCHAR(255) NOT NULL,
        UNIQUE (name, value)
    );

CREATE TABLE
    IF NOT EXISTS common_labels (
        id SERIAL PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
        value VARCHAR(255) NOT NULL,
        UNIQUE (name, value)
    );

CREATE TABLE
    IF NOT EXISTS common_annotations (
        id SERIAL PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
        value VARCHAR(255) NOT NULL,
        UNIQUE (name, value)
    );

CREATE TABLE
    IF NOT EXISTS groups_labels (
        group_id INTEGER NOT NULL references groups(id),
        label_id INTEGER NOT NULL references labels(id),
        PRIMARY KEY (group_id, label_id)
    );

CREATE TABLE
    IF NOT EXISTS groups_common_labels (
        group_id INTEGER NOT NULL references groups(id),
        common_label_id INTEGER NOT NULL references common_labels(id),
        PRIMARY KEY (group_id, common_label_id)
    );

CREATE TABLE
    IF NOT EXISTS groups_common_annotations (
        group_id INTEGER NOT NULL references groups(id),
        common_annotation_id INTEGER NOT NULL references common_annotations(id),
        PRIMARY KEY (
            group_id,
            common_annotation_id
        )
    );

CREATE TABLE
    IF NOT EXISTS alerts_labels (
        alert_id INTEGER NOT NULL references alerts(id),
        label_id INTEGER NOT NULL references labels(id),
        PRIMARY KEY (alert_id, label_id)
    );

CREATE TABLE
    IF NOT EXISTS alerts_annotations (
        alert_id INTEGER NOT NULL references alerts(id),
        annotation_id INTEGER NOT NULL references annotations(id),
        PRIMARY KEY (alert_id, annotation_id)
    );