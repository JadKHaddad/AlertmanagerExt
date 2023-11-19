-- Your SQL goes here
CREATE TYPE alert_status AS ENUM ('resolved', 'firing');

CREATE TABLE alert_group (
    id SERIAL PRIMARY KEY,
    timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
    group_key VARCHAR(255) NOT NULL,
    receiver VARCHAR(255) NOT NULL,
    status alert_status NOT NULL,
    external_url VARCHAR(255) NOT NULL
);

CREATE INDEX ON alert_group (timestamp);

CREATE INDEX ON alert_group (status, timestamp);

CREATE TABLE group_label (
    id SERIAL PRIMARY KEY,
    alert_group_id INTEGER NOT NULL references alert_group(id),
    name VARCHAR(255) NOT NULL,
    value VARCHAR(255) NOT NULL
);

CREATE TABLE common_label (
    id SERIAL PRIMARY KEY,
    alert_group_id INTEGER NOT NULL references alert_group(id),
    name VARCHAR(255) NOT NULL,
    value VARCHAR(255) NOT NULL
);

CREATE TABLE common_annotation (
    id SERIAL PRIMARY KEY,
    alert_group_id INTEGER NOT NULL references alert_group(id),
    name VARCHAR(255) NOT NULL,
    value VARCHAR(255) NOT NULL
);

CREATE TABLE alert (
    id SERIAL PRIMARY KEY,
    alert_group_id INTEGER NOT NULL references alert_group(id),
    status alert_status NOT NULL,
    starts_at TIMESTAMP NOT NULL,
    ends_at TIMESTAMP,
    generator_url VARCHAR(255) NOT NULL,
    fingerprint VARCHAR(255) NOT NULL
);

CREATE INDEX ON alert (starts_at);

CREATE INDEX ON alert (status, starts_at);

CREATE TABLE alert_label (
    id SERIAL PRIMARY KEY,
    alert_id INTEGER NOT NULL references alert(id),
    name VARCHAR(255) NOT NULL,
    value VARCHAR(255) NOT NULL
);

CREATE TABLE alert_annotation (
    id SERIAL PRIMARY KEY,
    alert_id INTEGER NOT NULL references alert(id),
    name VARCHAR(255) NOT NULL,
    value VARCHAR(255) NOT NULL
);