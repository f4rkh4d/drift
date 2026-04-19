# drift.security.public-schema

**write to public schema**  —  category: `security`  severity: `info`  fixable: false

run `drift explain drift.security.public-schema` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "security" rule, fires at `info` by default.

## configuration

```toml
[rules]
"drift.security.public-schema" = "error"     # or warning, info, off
```


