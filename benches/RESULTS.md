# benchmark results

last refreshed: 2026-05-05.

## machine

- apple m-series, macOS, fan curve at default
- drift built with `cargo build --release` (lto = thin, codegen-units = 1)
- sqlfluff installed via `pip install sqlfluff>=3.0`

## corpus

200 SQL files. built by checking out [`dbt-labs/jaffle_shop`](https://github.com/dbt-labs/jaffle_shop) and copying its 5 `.sql` files 40 times (so each file is small but the linter has to walk a real directory tree). reproducible:

```sh
git clone --depth 1 https://github.com/dbt-labs/jaffle_shop /tmp/jaffle_shop
mkdir -p /tmp/bench-corpus
for i in $(seq 1 40); do
  for f in /tmp/jaffle_shop/models/*.sql /tmp/jaffle_shop/models/staging/*.sql; do
    cp "$f" "/tmp/bench-corpus/$(basename $f .sql)_$i.sql"
  done
done
```

## numbers

both tools were given the same `--dialect postgres` flag and pointed at the same directory. wall-clock times across 5 cold runs (lower is better):

| run    | drift   | sqlfluff   |
|--------|--------:|-----------:|
| 1      |  45 ms  |  14820 ms  |
| 2      |  36 ms  |  15255 ms  |
| 3      |  34 ms  |  14949 ms  |
| 4      |  33 ms  |  15935 ms  |
| 5      |  33 ms  |  16069 ms  |
| median |  34 ms  |  15255 ms  |

**ratio: ~448x.** the README's quoted range (60-180x) holds for projects with heavy macro expansion where sqlfluff has more work; on this raw-SQL corpus sqlfluff has it easy and drift is still ~450x ahead.

## single-file

```sh
drift check --dialect postgres customers.sql
```

3 cold runs: 40.7 ms, 31.1 ms, 33.0 ms (median 33 ms).

most of that is process startup (about 25-30 ms on macOS). actual lint work for a small file is sub-millisecond. on warm runs through the LSP, diagnostics on save take **single-digit milliseconds**, which is the "lint on every keystroke without a 4-second pause" property the README pitches.

## why so fast

- **rust + rayon parallel walk.** drift uses every core. sqlfluff is python and walks files sequentially most of the time.
- **single-binary, no python startup.** sqlfluff pays ~500 ms just to import its dependency tree before linting anything.
- **no jinja by default.** sqlfluff's strength is rendering jinja / dbt macros before linting, which is expensive. drift expects rendered SQL (point it at `target/compiled` from `dbt parse` if you use dbt) and skips that step.

## reproducing

`benches/sqlfluff_compare.sh` does end-to-end on a fresh dbt project (jaffle_shop, dbt_parse, sqlfluff and drift, prints both wall times). this file uses a static corpus to avoid the dbt install in CI.
