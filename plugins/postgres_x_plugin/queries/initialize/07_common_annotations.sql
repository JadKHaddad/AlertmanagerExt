CREATE TABLE
    common_annotations (
        id SERIAL PRIMARY KEY,
        name VARCHAR(255) NOT NULL,
        value VARCHAR(255) NOT NULL,
        UNIQUE (name, value)
    );