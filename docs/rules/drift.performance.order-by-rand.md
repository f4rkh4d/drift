# drift.performance.order-by-rand

**ORDER BY random()**  —  category: `performance`  severity: `warning`  fixable: false

run `drift explain drift.performance.order-by-rand` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "performance" rule, fires at `warning` by default.

## configuration

```toml
[rules]
"drift.performance.order-by-rand" = "error"     # or warning, info, off
```


