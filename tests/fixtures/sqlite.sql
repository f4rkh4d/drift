-- sqlite-shaped queries.
CREATE TABLE notes (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  body TEXT NOT NULL,
  created_at INTEGER NOT NULL
);

INSERT INTO notes (body, created_at) VALUES ('hi', strftime('%s','now'));

SELECT id, body FROM notes ORDER BY created_at DESC LIMIT 20;
