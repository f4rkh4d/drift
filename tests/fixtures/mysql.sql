-- mysql-shaped queries.
SELECT `id`, `name` FROM `users` WHERE `active` = 1;

INSERT INTO counters (k, n) VALUES ('x', 1)
ON DUPLICATE KEY UPDATE n = n + 1;

SELECT id FROM users WHERE email LIKE '%@example.com';

CREATE TABLE orders (
  id BIGINT UNSIGNED AUTO_INCREMENT PRIMARY KEY,
  total DECIMAL(10,2) NOT NULL,
  created_at DATETIME NOT NULL
);
