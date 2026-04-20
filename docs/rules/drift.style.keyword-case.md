# drift.style.keyword-case

**keyword case** .  category: `style`  severity: `warning`  fixable: true

run `drift explain drift.style.keyword-case` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "style" rule, fires at `warning` by default.

## configuration

```toml
[rules]
"drift.style.keyword-case" = "error"     # or warning, info, off
```

## fix

`drift fix` will rewrite this automatically.
