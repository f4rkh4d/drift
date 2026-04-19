# drift.correctness.cartesian-join

**probable cartesian product**  —  category: `correctness`  severity: `warning`  fixable: false

run `drift explain drift.correctness.cartesian-join` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "correctness" rule, fires at `warning` by default.

## configuration

```toml
[rules]
"drift.correctness.cartesian-join" = "error"     # or warning, info, off
```


