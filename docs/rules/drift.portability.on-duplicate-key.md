# drift.portability.on-duplicate-key

**ON DUPLICATE KEY UPDATE** .  category: `portability`  severity: `info`  fixable: false

run `drift explain drift.portability.on-duplicate-key` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "portability" rule, fires at `info` by default.

## configuration

```toml
[rules]
"drift.portability.on-duplicate-key" = "error"     # or warning, info, off
```


