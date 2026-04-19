# drift.performance.fn-on-column

**function on column in WHERE**  —  category: `performance`  severity: `info`  fixable: false

run `drift explain drift.performance.fn-on-column` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "performance" rule, fires at `info` by default.

## configuration

```toml
[rules]
"drift.performance.fn-on-column" = "error"     # or warning, info, off
```


