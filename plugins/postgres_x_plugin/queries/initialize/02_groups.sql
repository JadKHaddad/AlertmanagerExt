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