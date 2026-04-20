# drift.conventions.pk-column-id

**primary key column name** .  category: `conventions`  severity: `info`  fixable: false

run `drift explain drift.conventions.pk-column-id` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "conventions" rule, fires at `info` by default.

## configuration

```toml
[rules]
"drift.conventions.pk-column-id" = "error"     # or warning, info, off
```


