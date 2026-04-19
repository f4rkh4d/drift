# drift.style.trailing-newline

**final newline**  —  category: `style`  severity: `info`  fixable: true

run `drift explain drift.style.trailing-newline` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "style" rule, fires at `info` by default.

## configuration

```toml
[rules]
"drift.style.trailing-newline" = "error"     # or warning, info, off
```

## fix

`drift fix` will rewrite this automatically.
