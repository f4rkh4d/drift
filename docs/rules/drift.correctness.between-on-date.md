# drift.correctness.between-on-date

**BETWEEN on dates is inclusive of the upper bound** .  category: `correctness`  severity: `warning`  fixable: false

run `drift explain drift.correctness.between-on-date` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "correctness" rule, fires at `warning` by default.

## configuration

```toml
[rules]
"drift.correctness.between-on-date" = "error"     # or warning, info, off
```


