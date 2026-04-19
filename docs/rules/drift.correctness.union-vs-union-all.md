# drift.correctness.union-vs-union-all

**UNION vs UNION ALL**  —  category: `correctness`  severity: `info`  fixable: false

run `drift explain drift.correctness.union-vs-union-all` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "correctness" rule, fires at `info` by default.

## configuration

```toml
[rules]
"drift.correctness.union-vs-union-all" = "error"     # or warning, info, off
```


