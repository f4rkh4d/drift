# drift.portability.pg-limit-offset

**LIMIT n OFFSET m**  —  category: `portability`  severity: `info`  fixable: false

run `drift explain drift.portability.pg-limit-offset` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "portability" rule, fires at `info` by default.

## configuration

```toml
[rules]
"drift.portability.pg-limit-offset" = "error"     # or warning, info, off
```


