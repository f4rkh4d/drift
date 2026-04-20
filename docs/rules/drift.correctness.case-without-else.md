# drift.correctness.case-without-else

**CASE without ELSE** .  category: `correctness`  severity: `info`  fixable: false

run `drift explain drift.correctness.case-without-else` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "correctness" rule, fires at `info` by default.

## configuration

```toml
[rules]
"drift.correctness.case-without-else" = "error"     # or warning, info, off
```


