CREATE TABLE sessions (
    id SERIAL PRIMARY KEY,
    start double precision NOT NULL,
    duration INTEGER NOT NULL,
    topic_id INTEGER REFERENCES topics(id)
)