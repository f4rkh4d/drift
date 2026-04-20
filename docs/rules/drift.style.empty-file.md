# drift.style.empty-file

**empty file** .  category: `style`  severity: `info`  fixable: false

run `drift explain drift.style.empty-file` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "style" rule, fires at `info` by default.

## configuration

```toml
[rules]
"drift.style.empty-file" = "error"     # or warning, info, off
```


