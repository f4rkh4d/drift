# drift.ambiguity.mixed-bool

**inconsistent TRUE/FALSE casing** .  category: `ambiguity`  severity: `info`  fixable: false

run `drift explain drift.ambiguity.mixed-bool` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "ambiguity" rule, fires at `info` by default.

## configuration

```toml
[rules]
"drift.ambiguity.mixed-bool" = "error"     # or warning, info, off
```


