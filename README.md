# drift

[![ci](https://github.com/f4rkh4d/drift/actions/workflows/ci.yml/badge.svg)](https://github.com/f4rkh4d/drift/actions/workflows/ci.yml)
[![crates.io](https://img.shields.io/crates/v/drift-sql.svg)](https://crates.io/crates/drift-sql)
[![release](https://img.shields.io/github/v/release/f4rkh4d/drift)](https://github.com/f4rkh4d/drift/releases)
[![license](https://img.shields.io/github/license/f4rkh4d/drift)](LICENSE)

![demo](docs/demo.gif)
sql linter and formatter in rust. 5 dialects. single binary.

80+ rules. on a 200-file SQL corpus drift's median wall time is **34 ms** vs sqlfluff's **15.3 s** (~448x faster). on a 4,200-file dbt project the ratio narrows to 60-180x because sqlfluff's heavy macro work pulls less from python overhead. full numbers and method in [`benches/RESULTS.md`](benches/RESULTS.md). reproduce: `bash benches/sqlfluff_compare.sh`.

```
$ drift check migrations/
migrations/0042_backfill.sql:12:1 error [drift.correctness.missing-where-update]
  UPDATE statement has no WHERE clause
migrations/0042_backfill.sql:12:1 warning [drift.performance.select-star]
  avoid SELECT *; list the columns you need
migrations/0044_seed.sql:3:12 error [drift.correctness.null-equality]
  comparing to NULL with = or <>; use IS NULL / IS NOT NULL
```

## install

```sh
# one-liner (linux + mac, amd64 + arm64)
curl -fsSL https://raw.githubusercontent.com/f4rkh4d/drift/main/install.sh | sh

# homebrew (mac)
brew install f4rkh4d/tap/drift

# cargo. note: the unscoped `drift` name on crates.io is an unrelated openapi
# tool; this crate ships as `drift-sql`, the installed binary is still `drift`.
cargo install drift-sql
```

pre-built binaries for linux/mac (amd64 + arm64) are attached to every release: <https://github.com/f4rkh4d/drift/releases>.

## quick start

```
drift check **/*.sql                       # lint, exit 1 on errors
drift check --fail-on warning ...          # exit 1 on warnings or errors
drift check --format sarif ...             # output for github code scanning
drift check --format json ...              # output for any other consumer
drift check --baseline .drift-baseline.json ...  # silence violations recorded in the baseline
drift check --watch migrations/            # re-lint on every save
drift baseline create migrations/          # snapshot current violations into .drift-baseline.json
drift baseline show                        # print a summary of an existing baseline
drift profile migrations/                  # which rules fire most, what to disable
drift docs                                 # regenerate docs/rules/<id>.md from rule trait
drift docs --check                         # ci gate: fail if docs are stale
drift fix                                  # apply safe auto-fixes
drift format queries.sql                   # reformat to stdout
drift format -i queries.sql                # rewrite in place
drift rules                                # list all rules
drift explain drift.correctness.null-equality
drift lsp                                  # language server over stdio
```

any of them take `--dialect postgres|mysql|sqlite|bigquery|ansi`. when left unset, drift looks at the file extension and then the nearest `drift.toml`.

## dialects

- **postgres**. primary. about 95% coverage in my test corpus.
- **mysql / mariadb**. merged. around 80%. some of the wilder index hints don't parse.
- **sqlite**. ~80%. `WITHOUT ROWID` works, `STRICT` is partial.
- **bigquery**. ~60%. struct/array literals parse, scripting blocks (`DECLARE`/`BEGIN`) mostly don't.
- **snowflake**. new in 0.15.0. `LATERAL FLATTEN`, named arguments (`input =>`), QUALIFY, `:variant.path` accessors. roughly 70%; multi-statement scripts and stored procedures still partial.
- **ansi**. baseline, for when you're writing for portability.

not shipped yet: tsql, redshift, oracle. on the roadmap. if you only use those, drift won't help you today.

## rule catalog

run `drift rules` for the full list. the short version:

| category | count | examples |
|---|---|---|
| style | 20 | keyword-case, indent, trailing-whitespace, leading-comma |
| correctness | 15 | missing-where-update, null-equality, cartesian-join |
| performance | 8 | select-star, like-leading-wildcard, offset-paging |
| security | 6 | plaintext-password, grant-all, drop-without-if-exists |
| portability | 8 | backtick-quote, on-duplicate-key, top-vs-limit |
| conventions | 8 | snake-case-tables, plural-table-name, index-naming |
| ambiguity | 5 | mixed-bool, reserved-as-identifier |

per-rule markdown pages live in `docs/rules/`.

## config

drop a `drift.toml` at your repo root (drift walks up looking for one):

```toml
[drift]
dialect = "postgres"
preset  = "strict"       # strict | warn | compat

[rules]
"drift.style.keyword-case"        = { severity = "warning", case = "upper" }
"drift.performance.select-star"   = "error"
"drift.correctness.*"             = "error"
"drift.portability.*"             = "off"

[format]
indent       = 2
max-line     = 100
keyword-case = "upper"
```

severity: `error` | `warning` | `info` | `off`. wildcards work at the leaf: `drift.correctness.*`.

## vs sqlfluff

|  | drift | sqlfluff |
|---|---|---|
| language | rust | python |
| binary | single static binary | pip install + deps |
| speed | seconds on a big dbt repo | minutes |
| memory | ~20mb | much more |
| dialects | 5 | 12+ |
| rule count | 80+ | 60+ |
| jinja / dbt macros | no | yes |
| plugin system | no | yes |
| lsp | yes | no |

short version: sqlfluff is broader, drift is faster. if your pipeline already runs sqlfluff against unrendered jinja, stay there. if it runs against rendered sql, drift's probably what you want.

## vs pgformatter, sqlfmt

pgformatter is postgres-only, formatter-only. sqlfmt is postgres + dbt-flavored, opinionated one-true-style. drift lints first, formats second, and spans more dialects. pick sqlfmt if you already live in that formatting style and don't care about lint rules.

## ci

### github actions

```yaml
- uses: f4rkh4d/drift@main
  with:
    paths: 'migrations/'
    fail-on: error
```

inputs: `command` (default `check`), `paths`, `fail-on` (`error|warning|info|never`), `config`, `version`, `args`. pin `@main` to a release tag for stability.

### github code scanning (sarif)

`--format sarif` produces SARIF 2.1.0 that github ingests natively. each violation surfaces as an inline annotation on the pull request that opened it.

```yaml
name: drift
on: [pull_request]
permissions:
  contents: read
  security-events: write
jobs:
  drift:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: f4rkh4d/drift@main
        with:
          paths: 'migrations/'
          args: '--format sarif'
          fail-on: never
        continue-on-error: true
        id: drift
      - uses: github/codeql-action/upload-sarif@v3
        with:
          sarif_file: drift.sarif
```

(piping drift's stdout to `drift.sarif` and uploading is the easy way; the action's `args: '--format sarif'` makes drift emit it. `fail-on: never` keeps the job from failing on findings so the upload always runs.)

### pre-commit

```yaml
# .pre-commit-config.yaml
repos:
  - repo: https://github.com/f4rkh4d/drift
    rev: v0.14.43
    hooks:
      - id: drift-check
      - id: drift-fix      # apply the safe auto-fixes
      - id: drift-format   # format in place
```

drift's pre-commit hooks invoke the binary on your `$PATH`, so install drift first via brew, the install script, or cargo.

## editor setup

vscode: scaffold lives at [`editors/vscode`](editors/vscode). marketplace publish is queued.

helix. in `languages.toml`:

```toml
[[language]]
name = "sql"
language-servers = ["drift"]

[language-server.drift]
command = "drift"
args = ["lsp"]
```

neovim (nvim-lspconfig):

```lua
require('lspconfig.configs').drift = {
  default_config = {
    cmd = { 'drift', 'lsp' },
    filetypes = { 'sql' },
    root_dir = require('lspconfig.util').root_pattern('drift.toml', '.git'),
  }
}
require('lspconfig').drift.setup{}
```

## profiling your codebase

`drift profile` runs a check and aggregates the results: which rules fire most, where the noise lives. it ends with a recommended `drift.toml` snippet for the rules that are over 5% of all hits.

```sh
$ drift profile migrations/
scanned 200 files in 7 ms
total violations: 6680
by severity:     6680 warning

top 5 rules:
  count  severity    rule
   6400  warning     drift.style.keyword-case
    120  warning     drift.style.identifier-case
     40  warning     drift.style.double-blank-line
     40  warning     drift.style.indent
     40  warning     drift.style.line-length

suggested drift.toml (rules above 5% of all hits, demoted to warning):

  [rules]
  "drift.style.keyword-case" = "warning"   # 6400 hits
```

`--json` for tool integration, `--top N` to widen the list.

## editor: vscode

a vscode extension lives at [`editors/vscode`](editors/vscode). it spawns `drift lsp` and surfaces diagnostics, code actions, and format-on-save inside the editor. install drift first via brew or the one-liner. marketplace publish is queued.

## adopting drift on a legacy codebase (baseline)

running drift cold against a 10-year-old SQL repo emits thousands of warnings, nobody fixes them, the team turns drift off. the baseline file fixes this:

```sh
# one-time: take a snapshot of every current violation.
drift baseline create migrations/ models/ analytics/

# from now on, only NEW violations surface. the legacy debt is locked.
drift check --baseline .drift-baseline.json migrations/ models/ analytics/
```

how it works:

- `.drift-baseline.json` records, for each file, the count of violations per rule. line numbers are not part of the key on purpose: code edits shift them.
- on subsequent `drift check --baseline`, the first N matching violations per (file, rule) are silenced. the (N+1)th and beyond surface.
- adding a new file with a violation surfaces normally. introducing a NEW rule violation in an old file surfaces normally.
- the summary line tells you how many were suppressed, so you always know how much debt remains: `... (147 suppressed by baseline)`.
- to refresh the baseline (e.g. after a cleanup pass): rerun `drift baseline create`.

commit `.drift-baseline.json` to your repo. inspect with `drift baseline show`.

## faq

**why rust?** because sqlfluff takes 4 minutes on our dbt project. this one takes 3 seconds. the rest is nice-to-have.

**will it replace sqlfluff?** for most projects, yes. if you need the plugin system or custom jinja templating, stay on sqlfluff. that's not what drift is.

**5 dialects? really all 5?** postgres is 95% coverage. mysql and sqlite are 80-ish. bigquery is 60. ansi is just a baseline. it'll improve.

**why is the binary 8mb if it's written in rust?** sqlparser vendored + clap + the whole serde stack + the lsp bits. every byte paid for. strip + thin-lto gets it under 6mb on mac arm64.

**is fix mode safe?** it only touches whitespace, keyword case, final newline, and trailing semicolons. it won't rewrite a query. i don't trust myself with anything semantic and you shouldn't either.

**does it support dbt?** not as a first-class thing. run dbt compile and point drift at `target/compiled`.

## roadmap

- snowflake, oracle, tsql, redshift dialects
- dbt project integration (read `dbt_project.yml`, respect `{{ ref() }}`)
- per-rule config via `-- drift: disable drift.style.keyword-case` comments
- rule sdk so teams can add their own checks without forking
- hover + completion in lsp (0.15)

## license

MIT. see LICENSE.

## reporting bugs

open an issue at [github.com/f4rkh4d/drift/issues](https://github.com/f4rkh4d/drift/issues) with a minimal reproduction.
