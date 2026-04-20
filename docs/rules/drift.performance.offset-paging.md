# drift.performance.offset-paging

**OFFSET for paging** .  category: `performance`  severity: `info`  fixable: false

run `drift explain drift.performance.offset-paging` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "performance" rule, fires at `info` by default.

## configuration

```toml
[rules]
"drift.performance.offset-paging" = "error"     # or warning, info, off
```


