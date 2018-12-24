-- Your SQL goes here
CREATE TABLE contacts (
    id SERIAL PRIMARY KEY,
    tourist_name VARCHAR NOT NULL,
    email VARCHAR NOT NULL,
    message VARCHAR NOT NULL,
    committed_time timestamp
)