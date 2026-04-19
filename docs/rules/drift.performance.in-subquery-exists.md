# drift.performance.in-subquery-exists

**IN (subquery) vs EXISTS**  —  category: `performance`  severity: `info`  fixable: false

run `drift explain drift.performance.in-subquery-exists` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "performance" rule, fires at `info` by default.

## configuration

```toml
[rules]
"drift.performance.in-subquery-exists" = "error"     # or warning, info, off
```


