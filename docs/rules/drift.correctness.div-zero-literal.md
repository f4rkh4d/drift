# drift.correctness.div-zero-literal

**literal division by zero** .  category: `correctness`  severity: `error`  fixable: false

run `drift explain drift.correctness.div-zero-literal` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "correctness" rule, fires at `error` by default.

## configuration

```toml
[rules]
"drift.correctness.div-zero-literal" = "error"     # or warning, info, off
```


