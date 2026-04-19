# drift.correctness.duplicate-column

**duplicate column in select**  —  category: `correctness`  severity: `warning`  fixable: false

run `drift explain drift.correctness.duplicate-column` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "correctness" rule, fires at `warning` by default.

## configuration

```toml
[rules]
"drift.correctness.duplicate-column" = "error"     # or warning, info, off
```


