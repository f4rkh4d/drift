# drift.ambiguity.duplicate-alias

**duplicate alias in same query** .  category: `ambiguity`  severity: `error`  fixable: false

run `drift explain drift.ambiguity.duplicate-alias` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "ambiguity" rule, fires at `error` by default.

## configuration

```toml
[rules]
"drift.ambiguity.duplicate-alias" = "error"     # or warning, info, off
```


