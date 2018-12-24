-- Your SQL goes here
CREATE TABLE posts
(
    id SERIAL PRIMARY KEY,
    title character varying(250) NOT NULL,
    slug character varying(250) NOT NULL,
    body text NOT NULL,
    publish timestamp,
    created timestamp,
    updated timestamp,
    status character varying(10) NOT NULL,
    user_id integer NOT NULL,
    likes integer NOT NULL,
    -- CONSTRAINT posts_pkey PRIMARY KEY (id),
    FOREIGN KEY (user_id) REFERENCES users(id)
    -- CONSTRAINT blog_post_author_id_dd7a8485_fk_auth_user_id FOREIGN KEY (author_id)
        -- REFERENCES users (id) MATCH SIMPLE
        -- ON UPDATE NO ACTION ON DELETE NO ACTION DEFERRABLE INITIALLY DEFERRED
)