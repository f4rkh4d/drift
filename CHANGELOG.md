# changelog

format loosely follows keep-a-changelog. dates are iso.

## [0.15.0]. 2026-05-05

### added

- one-line installer: `curl -fsSL https://raw.githubusercontent.com/f4rkh4d/drift/main/install.sh | sh`. picks the right linux/mac amd64/arm64 archive from the latest release.
- composite github action at the repo root (`action.yml`). pipelines can now `uses: f4rkh4d/drift@main` with `command`, `paths`, `fail-on`, `config`, `version`, `args` inputs.
- pre-commit hooks (`.pre-commit-hooks.yaml`): `drift-check`, `drift-fix`, `drift-format`.
- homebrew formula in `f4rkh4d/homebrew-tap`: `brew install f4rkh4d/tap/drift` on macOS.
- reproducible benchmark script at `benches/sqlfluff_compare.sh`. clones a public dbt project, renders it, runs drift and sqlfluff back to back, prints the two wall-clock times. lets readers verify the sqlfluff comparison instead of taking my word for it.
- crates.io / version / license / ci badges in the readme.

### changed

- crate renamed on crates.io from `drift` to `drift-sql`. the name `drift` was already taken by an unrelated openapi tool. the binary you get after `cargo install drift-sql` is still `drift`, so existing scripts and shell aliases keep working unchanged.

## [0.14.43]. 2026-04-19

### fixed

- `drift.performance.select-star` now respects `EXCLUDE` clauses in postgres 16+ projections (was firing on `SELECT * EXCLUDE (col)`)

## [0.14.42]. 2026-04-17

### fixed

- parse recovery on unterminated single-quoted string no longer panics. the tokenizer still produces the tokens it got, the parser returns an error, and style rules keep running.

## [0.14.41]. 2026-04-15

### fixed

- `drift.correctness.cartesian-join` stopped flagging `FROM (VALUES …) v1, (VALUES …) v2`

## [0.14.40]. 2026-04-14

### fixed

- lsp: diagnostics were cleared when the file had a parse error. now they're partial.

## [0.14.39]. 2026-04-13

### fixed

- `drift rules --json` output was missing the `fixable` field

## [0.14.38]. 2026-04-12

### fixed

- windows line endings in input are tolerated. the crlf rule still fires, but nothing panics.

## [0.14.37]. 2026-04-11

### fixed

- `drift fix --check` exits 1 only when there are actual changes (was exiting 1 even on no-op)

## [0.14.36]. 2026-04-10

### fixed

- `drift.portability.on-duplicate-key` was firing inside mysql fixtures (dialect check was inverted)

## [0.14.35]. 2026-04-09

### fixed

- memory spike on files over 20k lines. rayon chunking was holding whole-file strings per worker.

## [0.14.34]. 2026-04-08

### fixed

- `--dialect bq` resolved to ansi on case-sensitive comparisons

## [0.14.33]. 2026-04-07

### fixed

- `drift explain` rendered the header in bold even with `--no-color`

## [0.14.32]. 2026-04-06

### fixed

- empty `drift.toml` no longer errors. it reads as defaults.

## [0.14.31]. 2026-04-05

### fixed

- `drift.style.space-after-comma` mis-flagged commas inside string literals

## [0.14.30]. 2026-04-04

### fixed

- `drift.security.plaintext-password` was matching `PASSWORD_HASH` columns

## [0.14.29]. 2026-04-04

### fixed

- `drift check` with `--format json` emitted invalid json when there were zero violations (was bare newline)

## [0.14.28]. 2026-04-03

### fixed

- `drift.correctness.union-vs-union-all` ran twice per UNION when nested

## [0.14.27]. 2026-04-03

### fixed

- glob expansion on windows-style backslashes

## [0.14.26]. 2026-04-02

### fixed

- regression: `drift format --in-place` truncated to zero bytes when the input had no statements (empty after comments)

## [0.14.25]. 2026-04-02

### fixed

- `drift.performance.offset-paging` threshold lowered from 10k to 1k and mentioned in the message

## [0.14.24]. 2026-04-01

### fixed

- `drift lsp` crashed on `shutdown` before `initialize`

## [0.14.23]. 2026-04-01

### fixed

- typo in `drift.correctness.missing-where-delete` message

## [0.14.22]. 2026-04-01

### fixed

- `--config` arg wasn't being honored when passed after the subcommand

## [0.14.21]. 2026-03-31

### fixed

- panic on files ending in a bare backslash under mysql dialect

## [0.14.20]. 2026-03-31

### fixed

- checkstyle output was missing the xml declaration

## [0.14.19]. 2026-03-31

### fixed

- `drift rules` output ordering was nondeterministic run-to-run

## [0.14.18]. 2026-03-30

### fixed

- `drift.style.line-length` counted bytes, not codepoints

## [0.14.17]. 2026-03-30

### fixed

- `drift.conventions.plural-table-name` fired on names ending in `_data`

## [0.14.16]. 2026-03-30

### fixed

- `drift.correctness.null-equality` skipped `!=` (only caught `=` and `<>`)

## [0.14.15]. 2026-03-29

### fixed

- parallel file processing kept files open under a file-descriptor-limited ci runner

## [0.14.14]. 2026-03-29

### fixed

- `drift format` added a final newline even to empty files

## [0.14.13]. 2026-03-29

### fixed

- `drift.style.crlf` ran on stdin when stdin was already normalized

## [0.14.12]. 2026-03-29

### fixed

- `drift.portability.backtick-quote` didn't fire on `CREATE TABLE` in ansi mode

## [0.14.11]. 2026-03-28

### fixed

- release binary for aarch64-linux was missing; workflow matrix had a typo

## [0.14.10]. 2026-03-28

### fixed

- `drift fix` could emit invalid utf-8 when a keyword fix collided with a multibyte identifier (overlap detection was off-by-one)

## [0.14.9]. 2026-03-28

### fixed

- crash on zero-length files

## [0.14.8]. 2026-03-28

### fixed

- `Severity::Off` wasn't being skipped in the lsp diagnostics path

## [0.14.7]. 2026-03-28

### fixed

- wildcard config keys like `drift.style.*` took precedence over exact keys (now reversed, exact wins)

## [0.14.6]. 2026-03-28

### fixed

- `drift explain` panicked on unknown rule id instead of printing a nice message

## [0.14.5]. 2026-03-28

### fixed

- `drift rules --json` was emitting compact json, not pretty

## [0.14.4]. 2026-03-28

### fixed

- readme link to rule docs was broken

## [0.14.3]. 2026-03-28

### fixed

- `rayon` caused ordering flakiness in tests; now the output sorts by path then line

## [0.14.2]. 2026-04-03

### fixed

- default severity of `drift.style.semicolon-terminator` was info; matches the docs now (warning)

## [0.14.1]. 2026-04-01

### fixed

- `drift --version` printed a stale string on homebrew builds

## [0.14.0]. 2026-03-28

### added

- `drift explain <rule-id>`. full rule description, examples, and fix info
- preset system via `drift.toml` `preset = "strict" | "warn" | "compat"`
- `--format json` and `--format checkstyle` output for ci pipelines
- `drift rules --json` for machine-readable rule listing

### changed

- rule ids normalized to `drift.<category>.<rule>` form. old `style.keyword_case` style ids no longer work; the migration is a find+replace in `drift.toml`.
- `drift check` output colorizes severity labels by default (opt out with `--no-color`)

### removed

- the 0.13.x `[profile]` section of `drift.toml` in favor of `[drift] preset = ...`

## [0.13.0]. 2026-03-04

### added

- basic language server over stdio (`drift lsp`)
- `textDocument/publishDiagnostics` from live rule output
- code actions for auto-fixable style rules

### known issues

- lsp doesn't do hover or completion yet. that's the 0.15 goal.

## [0.12.0]. 2026-02-08

### added

- `drift fix` applies safe rewrites: keyword case, trailing whitespace, trailing newline, trailing semicolon
- `drift fix --check` prints a unified diff without modifying files
- `FixStats` struct exposed from the library for integrators

### changed

- `drift format` now layers on top of the fixer (keyword case + spacing around commas)

## [0.11.0]. 2026-01-14

### added

- ambiguity rules: `drift.ambiguity.reserved-as-identifier`, `.duplicate-alias`, `.unqualified-column`, `.mixed-bool`, `.same-name-fn-col`

### fixed

- `drift.correctness.self-join-no-alias` now walks joins correctly (was missing right-hand side)

## [0.10.0]. 2025-12-10

### added

- `drift format` subcommand (pipe-through formatter)
- `[format]` section in `drift.toml`: `indent`, `max-line`, `keyword-case`

### changed

- token stream now preserves whitespace tokens, enabling comment-aware rules

## [0.9.0]. 2025-11-18

### added

- `bigquery` dialect (about 60% coverage)
- partitioned-date heuristics in `drift.correctness.between-on-date`

## [0.8.0]. 2025-10-28

### added

- convention rules: snake_case tables, plural tables, lowercase columns, index naming, hungarian-notation check

## [0.7.0]. 2025-10-06

### added

- portability category
- `drift.portability.backtick-quote`, `.on-duplicate-key`, `.top-vs-limit`, `.non-standard-type`, `.dialect-fn`, `.regex-op`
- `--dialect` override flag

## [0.6.0]. 2025-09-09

### added

- security category: grant-all, plaintext-password, public-schema, dynamic-sql-concat, drop-without-if-exists, truncate-no-cascade

## [0.5.0]. 2025-08-14

### added

- sqlite dialect
- `drift.correctness.distinct-on-no-order`

## [0.4.0]. 2025-07-22

### added

- mysql / mariadb dialect (merged)
- backtick-quoted identifier handling
- `drift.correctness.union-vs-union-all`

## [0.3.0]. 2025-06-18

### added

- performance category
- `drift.performance.select-star`, `.like-leading-wildcard`, `.fn-on-column`, `.nested-subquery`, `.order-by-rand`, `.offset-paging`

## [0.2.0]. 2025-05-20

### added

- correctness rules: missing-where-update, missing-where-delete, cartesian-join, null-equality, div-zero-literal, order-by-ordinal, case-without-else

## [0.1.0]. 2025-04-15

### added

- initial skeleton
- postgres parser via sqlparser
- five style rules: keyword-case, trailing-whitespace, trailing-newline, semicolon-terminator, indent
- `drift check` subcommand
