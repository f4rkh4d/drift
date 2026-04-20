# drift.security.drop-without-if-exists

**DROP without IF EXISTS** .  category: `security`  severity: `info`  fixable: false

run `drift explain drift.security.drop-without-if-exists` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "security" rule, fires at `info` by default.

## configuration

```toml
[rules]
"drift.security.drop-without-if-exists" = "error"     # or warning, info, off
```


