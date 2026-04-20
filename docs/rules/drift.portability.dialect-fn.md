# drift.portability.dialect-fn

**dialect-only function** .  category: `portability`  severity: `info`  fixable: false

run `drift explain drift.portability.dialect-fn` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "portability" rule, fires at `info` by default.

## configuration

```toml
[rules]
"drift.portability.dialect-fn" = "error"     # or warning, info, off
```


