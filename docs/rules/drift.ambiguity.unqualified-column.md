# drift.ambiguity.unqualified-column

**unqualified column in joined query**  —  category: `ambiguity`  severity: `info`  fixable: false

run `drift explain drift.ambiguity.unqualified-column` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "ambiguity" rule, fires at `info` by default.

## configuration

```toml
[rules]
"drift.ambiguity.unqualified-column" = "error"     # or warning, info, off
```


