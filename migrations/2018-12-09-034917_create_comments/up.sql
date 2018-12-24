-- Your SQL goes here
CREATE TABLE comments
(
    id SERIAL PRIMARY KEY,
    username character varying(250) NOT NULL,
    email character varying(250) NOT NULL,
    comment text NOT NULL,
    committed_time timestamp,
    post_id integer NOT NULL,
    FOREIGN KEY (post_id) REFERENCES posts(id)
)