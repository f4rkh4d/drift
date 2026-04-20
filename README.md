# drift


![demo](docs/hero.gif)
sql linter and formatter in rust. 5 dialects. single binary.

80+ rules. on my laptop it lints a 4,200-file dbt project in about 3.1 seconds, which is somewhere between 60 and 180 times faster than sqlfluff against the same corpus depending on how many macros you have.

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

```
cargo install drift
```

pre-built binaries for linux/mac (amd64 + arm64) are attached to every release. a homebrew tap and an install.sh are on the 0.15 list.

## quick start

```
drift check **/*.sql         # lint
drift fix                    # apply safe auto-fixes
drift format queries.sql     # reformat to stdout
drift format -i queries.sql  # rewrite in place
drift rules                  # list all rules
drift explain drift.correctness.null-equality
drift lsp                    # language server over stdio
```

any of them take `--dialect postgres|mysql|sqlite|bigquery|ansi`. when left unset, drift looks at the file extension and then the nearest `drift.toml`.

## dialects

- **postgres**. primary. about 95% coverage in my test corpus.
- **mysql / mariadb**. merged. around 80%. some of the wilder index hints don't parse.
- **sqlite**. ~80%. `WITHOUT ROWID` works, `STRICT` is partial.
- **bigquery**. ~60%. struct/array literals parse, scripting blocks (`DECLARE`/`BEGIN`) mostly don't.
- **ansi**. baseline, for when you're writing for portability.

not shipped yet: tsql, snowflake, redshift, oracle. on the roadmap. if you only use those, drift won't help you today.

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

## editor setup

vscode: there's no published extension yet. point a generic lsp client at `drift lsp`.

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
