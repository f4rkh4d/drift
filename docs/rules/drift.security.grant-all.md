# drift.security.grant-all

**GRANT ALL** .  category: `security`  severity: `warning`  fixable: false

run `drift explain drift.security.grant-all` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "security" rule, fires at `warning` by default.

## configuration

```toml
[rules]
"drift.security.grant-all" = "error"     # or warning, info, off
```


