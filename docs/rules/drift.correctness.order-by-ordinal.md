# drift.correctness.order-by-ordinal

**ORDER BY ordinal** .  category: `correctness`  severity: `warning`  fixable: false

run `drift explain drift.correctness.order-by-ordinal` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "correctness" rule, fires at `warning` by default.

## configuration

```toml
[rules]
"drift.correctness.order-by-ordinal" = "error"     # or warning, info, off
```


