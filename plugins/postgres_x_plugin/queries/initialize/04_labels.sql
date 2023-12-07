CREATE TABLE
    labels (
        id SERIAL PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
        value VARCHAR(255) NOT NULL,
        UNIQUE (name, value)
    );