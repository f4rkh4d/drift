# drift.performance.select-star

**SELECT ***  —  category: `performance`  severity: `warning`  fixable: false

run `drift explain drift.performance.select-star` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "performance" rule, fires at `warning` by default.

## configuration

```toml
[rules]
"drift.performance.select-star" = "error"     # or warning, info, off
```


