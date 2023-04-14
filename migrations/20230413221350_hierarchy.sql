CREATE TABLE hierarchy(
    id SERIAL PRIMARY KEY,
    parent INTEGER NOT NULL REFERENCES topics(id),
    child INTEGER NOT NULL REFERENCES topics(id),
    depth INTEGER NOT NULL
)