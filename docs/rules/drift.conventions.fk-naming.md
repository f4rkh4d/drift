# drift.conventions.fk-naming

**foreign key naming** .  category: `conventions`  severity: `info`  fixable: false

run `drift explain drift.conventions.fk-naming` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "conventions" rule, fires at `info` by default.

## configuration

```toml
[rules]
"drift.conventions.fk-naming" = "error"     # or warning, info, off
```


