# drift.security.plaintext-password

**plaintext password literal**  —  category: `security`  severity: `error`  fixable: false

run `drift explain drift.security.plaintext-password` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "security" rule, fires at `error` by default.

## configuration

```toml
[rules]
"drift.security.plaintext-password" = "error"     # or warning, info, off
```


