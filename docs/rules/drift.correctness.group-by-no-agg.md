# drift.correctness.group-by-no-agg

**GROUP BY with no aggregation** .  category: `correctness`  severity: `info`  fixable: false

run `drift explain drift.correctness.group-by-no-agg` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "correctness" rule, fires at `info` by default.

## configuration

```toml
[rules]
"drift.correctness.group-by-no-agg" = "error"     # or warning, info, off
```


