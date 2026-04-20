# drift.portability.backtick-quote

**backtick identifier** .  category: `portability`  severity: `warning`  fixable: false

run `drift explain drift.portability.backtick-quote` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "portability" rule, fires at `warning` by default.

## configuration

```toml
[rules]
"drift.portability.backtick-quote" = "error"     # or warning, info, off
```


