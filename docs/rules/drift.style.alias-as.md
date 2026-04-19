# drift.style.alias-as

**explicit AS for column aliases**  —  category: `style`  severity: `info`  fixable: false

run `drift explain drift.style.alias-as` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "style" rule, fires at `info` by default.

## configuration

```toml
[rules]
"drift.style.alias-as" = "error"     # or warning, info, off
```


