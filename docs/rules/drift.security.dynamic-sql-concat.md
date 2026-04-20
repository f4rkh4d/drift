# drift.security.dynamic-sql-concat

**dynamic sql concatenation marker** .  category: `security`  severity: `warning`  fixable: false

run `drift explain drift.security.dynamic-sql-concat` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "security" rule, fires at `warning` by default.

## configuration

```toml
[rules]
"drift.security.dynamic-sql-concat" = "error"     # or warning, info, off
```


