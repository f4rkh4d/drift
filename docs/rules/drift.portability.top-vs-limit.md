# drift.portability.top-vs-limit

**SELECT TOP**  —  category: `portability`  severity: `warning`  fixable: false

run `drift explain drift.portability.top-vs-limit` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "portability" rule, fires at `warning` by default.

## configuration

```toml
[rules]
"drift.portability.top-vs-limit" = "error"     # or warning, info, off
```


