-- Your SQL goes here
CREATE TABLE alert_group (
    id INTEGER PRIMARY KEY,
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    group_key VARCHAR(255) NOT NULL,
    receiver VARCHAR(255) NOT NULL,
    status VARCHAR(10) CHECK(status IN ('resolved', 'firing')) NOT NULL,
    external_url VARCHAR(255) NOT NULL
);

CREATE TABLE group_label (
    id INTEGER PRIMARY KEY,
    alert_group_id INTEGER NOT NULL references alert_group(id),
    name VARCHAR(255) NOT NULL,
    value VARCHAR(255) NOT NULL
);

CREATE TABLE common_label (
    id INTEGER PRIMARY KEY,
    alert_group_id INTEGER NOT NULL references alert_group(id),
    name VARCHAR(255) NOT NULL,
    value VARCHAR(255) NOT NULL
);

CREATE TABLE common_annotation (
    id INTEGER PRIMARY KEY,
    alert_group_id INTEGER NOT NULL references alert_group(id),
    name VARCHAR(255) NOT NULL,
    value VARCHAR(255) NOT NULL
);

CREATE TABLE alert (
    id INTEGER PRIMARY KEY,
    alert_group_id INTEGER NOT NULL references alert_group(id),
    status VARCHAR(10) CHECK(status IN ('resolved', 'firing')) NOT NULL,
    starts_at TIMESTAMP NOT NULL,
    ends_at TIMESTAMP,
    generator_url VARCHAR(255) NOT NULL,
    fingerprint VARCHAR(255) NOT NULL
);

CREATE TABLE alert_label (
    id INTEGER PRIMARY KEY,
    alert_id INTEGER NOT NULL references alert(id),
    name VARCHAR(255) NOT NULL,
    value VARCHAR(255) NOT NULL
);

CREATE TABLE alert_annotation (
    id INTEGER PRIMARY KEY,
    alert_id INTEGER NOT NULL references alert(id),
    name VARCHAR(255) NOT NULL,
    value VARCHAR(255) NOT NULL
);