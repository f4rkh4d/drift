# drift.style.trailing-whitespace

**trailing whitespace**  —  category: `style`  severity: `info`  fixable: true

run `drift explain drift.style.trailing-whitespace` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "style" rule, fires at `info` by default.

## configuration

```toml
[rules]
"drift.style.trailing-whitespace" = "error"     # or warning, info, off
```

## fix

`drift fix` will rewrite this automatically.
