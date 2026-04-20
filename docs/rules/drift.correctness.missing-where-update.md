# drift.correctness.missing-where-update

**UPDATE without WHERE** .  category: `correctness`  severity: `error`  fixable: false

run `drift explain drift.correctness.missing-where-update` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "correctness" rule, fires at `error` by default.

## configuration

```toml
[rules]
"drift.correctness.missing-where-update" = "error"     # or warning, info, off
```


