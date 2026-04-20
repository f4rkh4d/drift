# drift.correctness.implicit-coercion

**implicit type coercion** .  category: `correctness`  severity: `warning`  fixable: false

run `drift explain drift.correctness.implicit-coercion` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "correctness" rule, fires at `warning` by default.

## configuration

```toml
[rules]
"drift.correctness.implicit-coercion" = "error"     # or warning, info, off
```


