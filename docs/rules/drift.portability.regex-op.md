# drift.portability.regex-op

**regex operator** .  category: `portability`  severity: `info`  fixable: false

run `drift explain drift.portability.regex-op` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "portability" rule, fires at `info` by default.

## configuration

```toml
[rules]
"drift.portability.regex-op" = "error"     # or warning, info, off
```


