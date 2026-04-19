# drift.security.truncate-no-cascade

**TRUNCATE without explicit cascade/restrict**  —  category: `security`  severity: `info`  fixable: false

run `drift explain drift.security.truncate-no-cascade` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "security" rule, fires at `info` by default.

## configuration

```toml
[rules]
"drift.security.truncate-no-cascade" = "error"     # or warning, info, off
```


