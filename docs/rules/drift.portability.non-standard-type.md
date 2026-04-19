# drift.portability.non-standard-type

**non-standard type**  —  category: `portability`  severity: `info`  fixable: false

run `drift explain drift.portability.non-standard-type` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "portability" rule, fires at `info` by default.

## configuration

```toml
[rules]
"drift.portability.non-standard-type" = "error"     # or warning, info, off
```


