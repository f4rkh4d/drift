# drift.performance.nested-subquery

**nested subquery** .  category: `performance`  severity: `info`  fixable: false

run `drift explain drift.performance.nested-subquery` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "performance" rule, fires at `info` by default.

## configuration

```toml
[rules]
"drift.performance.nested-subquery" = "error"     # or warning, info, off
```


