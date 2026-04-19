# drift.style.reserved-word-quoted

**quoted reserved word identifier**  —  category: `style`  severity: `info`  fixable: false

run `drift explain drift.style.reserved-word-quoted` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "style" rule, fires at `info` by default.

## configuration

```toml
[rules]
"drift.style.reserved-word-quoted" = "error"     # or warning, info, off
```


