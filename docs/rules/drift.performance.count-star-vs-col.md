# drift.performance.count-star-vs-col

**COUNT(column) vs COUNT(*)** .  category: `performance`  severity: `info`  fixable: false

run `drift explain drift.performance.count-star-vs-col` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "performance" rule, fires at `info` by default.

## configuration

```toml
[rules]
"drift.performance.count-star-vs-col" = "error"     # or warning, info, off
```


