# rule index

71 rules.

## ambiguity

- [`drift.ambiguity.reserved-as-identifier`](./drift.ambiguity.reserved-as-identifier.md) — using a reserved word as an identifier forces quoting everywhere it appears
- [`drift.ambiguity.duplicate-alias`](./drift.ambiguity.duplicate-alias.md) — two tables with the same alias in one FROM clause is ambiguous
- [`drift.ambiguity.unqualified-column`](./drift.ambiguity.unqualified-column.md) — when two tables are joined, every column reference should be qualified
- [`drift.ambiguity.mixed-bool`](./drift.ambiguity.mixed-bool.md) — pick one: TRUE / true. don't mix.
- [`drift.ambiguity.same-name-fn-col`](./drift.ambiguity.same-name-fn-col.md) — naming a column `count` or `current_date` invites parse ambiguity

## conventions

- [`drift.conventions.snake-case-tables`](./drift.conventions.snake-case-tables.md) — table names should be snake_case
- [`drift.conventions.plural-table-name`](./drift.conventions.plural-table-name.md) — table names should be plural (e.g. users, not user)
- [`drift.conventions.upper-keywords`](./drift.conventions.upper-keywords.md) — the project convention is UPPERCASE keywords (duplicate of style.keyword-case for teams that prefer it here)
- [`drift.conventions.lowercase-columns`](./drift.conventions.lowercase-columns.md) — column names should be lowercase snake_case
- [`drift.conventions.pk-column-id`](./drift.conventions.pk-column-id.md) — primary key column should be `id` (not `user_id` inside users table)
- [`drift.conventions.fk-naming`](./drift.conventions.fk-naming.md) — foreign key columns should be `<referenced>_id`
- [`drift.conventions.index-naming`](./drift.conventions.index-naming.md) — index names should be `ix_<table>_<cols>`
- [`drift.conventions.no-hungarian`](./drift.conventions.no-hungarian.md) — don't prefix columns with type (e.g. `str_name`, `int_count`)

## correctness

- [`drift.correctness.missing-where-update`](./drift.correctness.missing-where-update.md) — UPDATE without WHERE rewrites every row. almost always a mistake.
- [`drift.correctness.missing-where-delete`](./drift.correctness.missing-where-delete.md) — DELETE without WHERE empties the table. use TRUNCATE if that's what you meant.
- [`drift.correctness.self-join-no-alias`](./drift.correctness.self-join-no-alias.md) — a table joined with itself needs aliases on both sides
- [`drift.correctness.cartesian-join`](./drift.correctness.cartesian-join.md) — multiple tables in FROM with no WHERE or JOIN predicate
- [`drift.correctness.between-on-date`](./drift.correctness.between-on-date.md) — BETWEEN '2025-01-01' AND '2025-01-31' excludes the last day of january when cast to timestamp
- [`drift.correctness.implicit-coercion`](./drift.correctness.implicit-coercion.md) — comparing a number column to a string literal ('5' vs 5) forces a coercion
- [`drift.correctness.case-without-else`](./drift.correctness.case-without-else.md) — CASE without ELSE returns NULL for unmatched rows — usually unintended
- [`drift.correctness.null-equality`](./drift.correctness.null-equality.md) — `x = NULL` is always unknown. use `x IS NULL`.
- [`drift.correctness.distinct-on-no-order`](./drift.correctness.distinct-on-no-order.md) — DISTINCT ON without a matching ORDER BY returns arbitrary rows
- [`drift.correctness.union-vs-union-all`](./drift.correctness.union-vs-union-all.md) — plain UNION deduplicates, which you rarely want. be explicit.
- [`drift.correctness.div-zero-literal`](./drift.correctness.div-zero-literal.md) — `/ 0` as a literal is a guaranteed runtime error
- [`drift.correctness.duplicate-column`](./drift.correctness.duplicate-column.md) — same column appearing twice in SELECT without aliasing
- [`drift.correctness.order-by-ordinal`](./drift.correctness.order-by-ordinal.md) — ORDER BY 1, 2 is fragile; use explicit column names
- [`drift.correctness.group-by-no-agg`](./drift.correctness.group-by-no-agg.md) — GROUP BY with no aggregate is equivalent to DISTINCT
- [`drift.correctness.reserved-fn-name`](./drift.correctness.reserved-fn-name.md) — CREATE FUNCTION with a reserved name will shadow built-ins

## performance

- [`drift.performance.select-star`](./drift.performance.select-star.md) — SELECT * fetches columns you don't need and breaks when the schema changes
- [`drift.performance.like-leading-wildcard`](./drift.performance.like-leading-wildcard.md) — LIKE '%foo' can't use a btree index. full table scan territory.
- [`drift.performance.fn-on-column`](./drift.performance.fn-on-column.md) — calling a function on a column in WHERE prevents the index from being used
- [`drift.performance.nested-subquery`](./drift.performance.nested-subquery.md) — deeply nested subqueries often rewrite to JOINs cleanly
- [`drift.performance.order-by-rand`](./drift.performance.order-by-rand.md) — ORDER BY random() / RAND() sorts the whole table to pick N rows
- [`drift.performance.count-star-vs-col`](./drift.performance.count-star-vs-col.md) — COUNT(col) filters nulls; if you want total rows, use COUNT(*)
- [`drift.performance.in-subquery-exists`](./drift.performance.in-subquery-exists.md) — IN (subquery) with a large result is often faster as EXISTS
- [`drift.performance.offset-paging`](./drift.performance.offset-paging.md) — OFFSET is O(n) in most engines; prefer keyset paging for deep pages

## portability

- [`drift.portability.backtick-quote`](./drift.portability.backtick-quote.md) — backticks are mysql/bigquery-only; ansi uses double quotes
- [`drift.portability.double-quote-ident`](./drift.portability.double-quote-ident.md) — double quotes are identifiers in ansi but strings in some mysql configs
- [`drift.portability.pg-limit-offset`](./drift.portability.pg-limit-offset.md) — LIMIT/OFFSET is postgres/mysql; ansi uses FETCH FIRST n ROWS ONLY
- [`drift.portability.on-duplicate-key`](./drift.portability.on-duplicate-key.md) — ON DUPLICATE KEY UPDATE is mysql-only; postgres has ON CONFLICT
- [`drift.portability.non-standard-type`](./drift.portability.non-standard-type.md) — types like SERIAL, DATETIME, TINYINT are dialect-specific
- [`drift.portability.dialect-fn`](./drift.portability.dialect-fn.md) — functions like GENERATE_SERIES, IFNULL, IF() are dialect-bound
- [`drift.portability.top-vs-limit`](./drift.portability.top-vs-limit.md) — SELECT TOP is tsql-only; drift doesn't support tsql yet
- [`drift.portability.regex-op`](./drift.portability.regex-op.md) — ~ and ~* are postgres-only; mysql uses REGEXP

## security

- [`drift.security.grant-all`](./drift.security.grant-all.md) — GRANT ALL is almost never what you want. specify the privileges.
- [`drift.security.public-schema`](./drift.security.public-schema.md) — public schema writes are an old postgres habit; scoped schemas are safer
- [`drift.security.plaintext-password`](./drift.security.plaintext-password.md) — password literals in migrations leak to logs, backups, git history
- [`drift.security.dynamic-sql-concat`](./drift.security.dynamic-sql-concat.md) — concatenated sql in stored procs is an injection smell
- [`drift.security.drop-without-if-exists`](./drift.security.drop-without-if-exists.md) — idempotent migrations should use IF EXISTS
- [`drift.security.truncate-no-cascade`](./drift.security.truncate-no-cascade.md) — TRUNCATE semantics differ across engines; state cascade/restrict explicitly
- [`drift.security.select-into-outfile`](./drift.security.select-into-outfile.md) — MySQL SELECT INTO OUTFILE/DUMPFILE writes to the server filesystem. often abused for SQLi-to-RCE pivots and leaks data outside DB access control.

## style

- [`drift.style.keyword-case`](./drift.style.keyword-case.md) — sql keywords should use a consistent case (upper by default)
- [`drift.style.identifier-case`](./drift.style.identifier-case.md) — unquoted identifiers should be lowercase
- [`drift.style.indent`](./drift.style.indent.md) — indentation should be a multiple of the configured indent width
- [`drift.style.trailing-whitespace`](./drift.style.trailing-whitespace.md) — lines should not end with whitespace
- [`drift.style.trailing-newline`](./drift.style.trailing-newline.md) — file should end with a single newline
- [`drift.style.semicolon-terminator`](./drift.style.semicolon-terminator.md) — every statement should end with a semicolon
- [`drift.style.leading-comma`](./drift.style.leading-comma.md) — flags lines that start with a comma when the style is trailing
- [`drift.style.double-blank-line`](./drift.style.double-blank-line.md) — no more than one consecutive blank line
- [`drift.style.tab-indent`](./drift.style.tab-indent.md) — tabs are forbidden for indentation
- [`drift.style.space-before-comma`](./drift.style.space-before-comma.md) — do not put whitespace before commas
- [`drift.style.space-after-comma`](./drift.style.space-after-comma.md) — commas must be followed by whitespace
- [`drift.style.space-around-operator`](./drift.style.space-around-operator.md) — binary operators should have whitespace on both sides
- [`drift.style.alias-as`](./drift.style.alias-as.md) — use explicit AS for column aliases (select a AS b, not select a b)
- [`drift.style.single-quote-string`](./drift.style.single-quote-string.md) — string literals should use single quotes (double quotes are identifiers in ansi sql)
- [`drift.style.reserved-word-quoted`](./drift.style.reserved-word-quoted.md) — avoid using reserved keywords as identifiers, even quoted
- [`drift.style.line-length`](./drift.style.line-length.md) — lines should be within the configured max length
- [`drift.style.redundant-parens`](./drift.style.redundant-parens.md) — flags `((expr))` — one level is enough
- [`drift.style.empty-file`](./drift.style.empty-file.md) — empty or whitespace-only files probably aren't intended
- [`drift.style.trailing-comma`](./drift.style.trailing-comma.md) — trailing commas before `from` / `)` are a parse error in most dialects
- [`drift.style.crlf`](./drift.style.crlf.md) — files should use LF line endings, not CRLF

