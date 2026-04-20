# drift.style.trailing-comma

**trailing comma in select list** .  category: `style`  severity: `warning`  fixable: false

run `drift explain drift.style.trailing-comma` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "style" rule, fires at `warning` by default.

## configuration

```toml
[rules]
"drift.style.trailing-comma" = "error"     # or warning, info, off
```


