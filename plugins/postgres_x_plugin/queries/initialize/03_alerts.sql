CREATE TABLE
    alerts (
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