-- ansi-style, nothing dialect-specific.
SELECT id, name FROM users WHERE active = TRUE FETCH FIRST 10 ROWS ONLY;

INSERT INTO audit_log (actor, action) VALUES ('system', 'boot');
