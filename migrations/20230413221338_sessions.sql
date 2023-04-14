CREATE TABLE sessions (
    id SERIAL PRIMARY KEY,
    start double precision NOT NULL,
    "end" double precision NOT NULL,
    topic_id INTEGER REFERENCES topics(id)
)