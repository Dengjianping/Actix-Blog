-- Your SQL goes here

CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    password VARCHAR NOT NULL,
    username VARCHAR NOT NULL,
    first_name VARCHAR NOT NULL,
    last_name VARCHAR NOT NULL,
    email VARCHAR NOT NULL,
    is_superuser boolean NOT NULL DEFAULT 'f',
    is_staff boolean NOT NULL DEFAULT 'f',
    is_active boolean NOT NULL DEFAULT 'f',
    last_login timestamp,
    date_joined timestamp
)