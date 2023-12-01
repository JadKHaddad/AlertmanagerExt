-- Your SQL goes here

CREATE TABLE
    alert_group (
        id INTEGER PRIMARY KEY,
        timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
        group_key VARCHAR(255) NOT NULL,
        receiver VARCHAR(255) NOT NULL,
        status VARCHAR(10) CHECK(
            status IN ('resolved', 'firing')
        ) NOT NULL,
        external_url VARCHAR(255) NOT NULL,
        UNIQUE (group_key)
    );

CREATE TABLE
    group_label (
        id INTEGER PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
        value VARCHAR(255) NOT NULL,
        UNIQUE (name, value)
    );

CREATE TABLE
    assign_group_label (
        id INTEGER PRIMARY KEY,
        alert_group_id INTEGER NOT NULL references alert_group(id),
        group_label_id INTEGER NOT NULL references group_label(id)
    );

CREATE TABLE
    common_label (
        id INTEGER PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
        value VARCHAR(255) NOT NULL,
        UNIQUE (name, value)
    );

CREATE TABLE
    assign_common_label(
        id INTEGER PRIMARY KEY,
        alert_group_id INTEGER NOT NULL references alert_group(id),
        common_label_id INTEGER NOT NULL references common_label(id)
    );

CREATE TABLE
    common_annotation (
        id INTEGER PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
        value VARCHAR(255) NOT NULL,
        UNIQUE (name, value)
    );

CREATE TABLE
    assign_common_annotation (
        id INTEGER PRIMARY KEY,
        alert_group_id INTEGER NOT NULL references alert_group(id),
        common_annotation_id INTEGER NOT NULL references common_annotation(id)
    );

CREATE TABLE
    alert (
        id INTEGER PRIMARY KEY,
        alert_group_id INTEGER NOT NULL references alert_group(id),
        group_key VARCHAR(255) NOT NULL references alert_group(group_key),
        status VARCHAR(10) CHECK(
            status IN ('resolved', 'firing')
        ) NOT NULL,
        starts_at TIMESTAMP NOT NULL,
        ends_at TIMESTAMP,
        generator_url VARCHAR(255) NOT NULL,
        fingerprint VARCHAR(255) NOT NULL,
        UNIQUE (group_key, fingerprint)
    );

CREATE TABLE
    alert_label (
        id INTEGER PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
        value VARCHAR(255) NOT NULL,
        UNIQUE (name, value)
    );

CREATE TABLE
    assign_alert_label (
        id INTEGER PRIMARY KEY,
        alert_id INTEGER NOT NULL references alert(id),
        alert_label_id INTEGER NOT NULL references alert_label(id)
    );

CREATE TABLE
    alert_annotation (
        id INTEGER PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
        value VARCHAR(255) NOT NULL,
        UNIQUE (name, value)
    );

CREATE TABLE
    assign_alert_annotation (
        id INTEGER PRIMARY KEY,
        alert_id INTEGER NOT NULL references alert(id),
        alert_annotation_id INTEGER NOT NULL references alert_annotation(id)
    );