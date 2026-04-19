# drift.correctness.self-join-no-alias

**self-join without alias**  —  category: `correctness`  severity: `error`  fixable: false

run `drift explain drift.correctness.self-join-no-alias` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "correctness" rule, fires at `error` by default.

## configuration

```toml
[rules]
"drift.correctness.self-join-no-alias" = "error"     # or warning, info, off
```


