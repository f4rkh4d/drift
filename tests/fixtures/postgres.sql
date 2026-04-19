-- a grab-bag of postgres patterns drift should understand.

select * from users where email like '%@example.com';

SELECT u.id, u.name, u.email
FROM users u
JOIN orders o ON u.id = o.user_id
WHERE o.status = 'paid'
  AND u.deleted_at IS NULL
ORDER BY u.id
LIMIT 100;

UPDATE users SET last_seen = now();

DELETE FROM sessions;

SELECT id FROM users WHERE id = '42';

CREATE TABLE Users (
  Id bigserial PRIMARY KEY,
  Email text NOT NULL
);

GRANT ALL ON users TO public;

SELECT DISTINCT ON (user_id) user_id, created_at FROM events;

SELECT id FROM users UNION SELECT id FROM deleted_users;

SELECT id FROM users WHERE lower(email) = 'x';

SELECT a / 0 FROM t;

SELECT id FROM users ORDER BY 1;

SELECT id FROM users u, orders o;

SELECT id FROM u, u;

SELECT id FROM users WHERE status = NULL;

SELECT ((id)) FROM users;

CREATE INDEX users_email ON users(email);
