# drift.correctness.reserved-fn-name

**function declared with reserved name** .  category: `correctness`  severity: `warning`  fixable: false

run `drift explain drift.correctness.reserved-fn-name` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "correctness" rule, fires at `warning` by default.

## configuration

```toml
[rules]
"drift.correctness.reserved-fn-name" = "error"     # or warning, info, off
```


