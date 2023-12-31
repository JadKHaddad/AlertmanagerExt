-- Add up migration script here

CREATE TABLE
    IF NOT EXISTS `groups` (
        id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
        timestamp TIMESTAMP NOT NULL DEFAULT CURRENT_TIMESTAMP,
        group_key VARCHAR(255) NOT NULL,
        receiver VARCHAR(255) NOT NULL,
        status VARCHAR(10) NOT NULL,
        external_url VARCHAR(255) NOT NULL,
        UNIQUE (group_key),
        CHECK (
            status IN ('resolved', 'firing')
        )
    );

CREATE TABLE
    IF NOT EXISTS alerts (
        id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
        group_id BIGINT UNSIGNED NOT NULL,
        group_key VARCHAR(255) NOT NULL,
        status VARCHAR(10) NOT NULL,
        starts_at TIMESTAMP NOT NULL,
        ends_at TIMESTAMP,
        generator_url VARCHAR(255) NOT NULL,
        fingerprint VARCHAR(255) NOT NULL,
        UNIQUE (group_key, fingerprint),
        FOREIGN KEY (group_id) REFERENCES `groups`(id),
        FOREIGN KEY (group_key) REFERENCES `groups`(group_key),
        CHECK (
            status IN ('resolved', 'firing')
        )
    );

CREATE TABLE
    IF NOT EXISTS labels (
        id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
        name VARCHAR(255) NOT NULL,
        value VARCHAR(255) NOT NULL,
        UNIQUE (name, value)
    );

CREATE TABLE
    IF NOT EXISTS annotations (
        id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
        name VARCHAR(255) NOT NULL,
        value VARCHAR(255) NOT NULL,
        UNIQUE (name, value)
    );

CREATE TABLE
    IF NOT EXISTS common_labels (
        id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
        name VARCHAR(255) NOT NULL,
        value VARCHAR(255) NOT NULL,
        UNIQUE (name, value)
    );

CREATE TABLE
    IF NOT EXISTS common_annotations (
        id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
        name VARCHAR(255) NOT NULL,
        value VARCHAR(255) NOT NULL,
        UNIQUE (name, value)
    );

CREATE TABLE
    IF NOT EXISTS groups_labels (
        id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
        group_id BIGINT UNSIGNED NOT NULL,
        label_id BIGINT UNSIGNED NOT NULL,
        FOREIGN KEY (group_id) REFERENCES `groups`(id),
        FOREIGN KEY (label_id) REFERENCES labels(id)
    );

CREATE TABLE
    IF NOT EXISTS groups_common_labels (
        id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
        group_id BIGINT UNSIGNED NOT NULL,
        common_label_id BIGINT UNSIGNED NOT NULL,
        FOREIGN KEY (group_id) REFERENCES `groups`(id),
        FOREIGN KEY (common_label_id) REFERENCES common_labels(id)
    );

CREATE TABLE
    IF NOT EXISTS groups_common_annotations (
        id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
        group_id BIGINT UNSIGNED NOT NULL,
        common_annotation_id BIGINT UNSIGNED NOT NULL,
        FOREIGN KEY (group_id) REFERENCES `groups`(id),
        FOREIGN KEY (common_annotation_id) REFERENCES common_annotations(id)
    );

CREATE TABLE
    IF NOT EXISTS alerts_labels (
        id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
        alert_id BIGINT UNSIGNED NOT NULL,
        label_id BIGINT UNSIGNED NOT NULL,
        FOREIGN KEY (alert_id) REFERENCES alerts(id),
        FOREIGN KEY (label_id) REFERENCES labels(id)
    );

CREATE TABLE
    IF NOT EXISTS alerts_annotations (
        id BIGINT UNSIGNED PRIMARY KEY AUTO_INCREMENT,
        alert_id BIGINT UNSIGNED NOT NULL,
        annotation_id BIGINT UNSIGNED NOT NULL,
        FOREIGN KEY (alert_id) REFERENCES alerts(id),
        FOREIGN KEY (annotation_id) REFERENCES annotations(id)
    );