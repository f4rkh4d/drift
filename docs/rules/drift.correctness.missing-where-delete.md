# drift.correctness.missing-where-delete

**DELETE without WHERE**  —  category: `correctness`  severity: `error`  fixable: false

run `drift explain drift.correctness.missing-where-delete` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "correctness" rule, fires at `error` by default.

## configuration

```toml
[rules]
"drift.correctness.missing-where-delete" = "error"     # or warning, info, off
```


