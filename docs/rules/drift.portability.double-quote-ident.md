# drift.portability.double-quote-ident

**double-quoted identifier** .  category: `portability`  severity: `info`  fixable: false

run `drift explain drift.portability.double-quote-ident` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "portability" rule, fires at `info` by default.

## configuration

```toml
[rules]
"drift.portability.double-quote-ident" = "error"     # or warning, info, off
```


