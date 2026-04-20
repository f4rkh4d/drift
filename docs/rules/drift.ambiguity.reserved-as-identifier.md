# drift.ambiguity.reserved-as-identifier

**reserved keyword as identifier** .  category: `ambiguity`  severity: `warning`  fixable: false

run `drift explain drift.ambiguity.reserved-as-identifier` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "ambiguity" rule, fires at `warning` by default.

## configuration

```toml
[rules]
"drift.ambiguity.reserved-as-identifier" = "error"     # or warning, info, off
```


