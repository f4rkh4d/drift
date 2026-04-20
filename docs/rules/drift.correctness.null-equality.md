# drift.correctness.null-equality

**= NULL instead of IS NULL** .  category: `correctness`  severity: `error`  fixable: false

run `drift explain drift.correctness.null-equality` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "correctness" rule, fires at `error` by default.

## configuration

```toml
[rules]
"drift.correctness.null-equality" = "error"     # or warning, info, off
```


