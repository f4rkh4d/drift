# drift.correctness.distinct-on-no-order

**DISTINCT ON without ORDER BY** .  category: `correctness`  severity: `warning`  fixable: false

run `drift explain drift.correctness.distinct-on-no-order` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "correctness" rule, fires at `warning` by default.

## configuration

```toml
[rules]
"drift.correctness.distinct-on-no-order" = "error"     # or warning, info, off
```


