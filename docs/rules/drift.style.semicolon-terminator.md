# drift.style.semicolon-terminator

**semicolon terminator** .  category: `style`  severity: `warning`  fixable: true

run `drift explain drift.style.semicolon-terminator` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "style" rule, fires at `warning` by default.

## configuration

```toml
[rules]
"drift.style.semicolon-terminator" = "error"     # or warning, info, off
```

## fix

`drift fix` will rewrite this automatically.
