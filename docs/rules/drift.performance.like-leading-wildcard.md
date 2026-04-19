# drift.performance.like-leading-wildcard

**LIKE with leading %**  —  category: `performance`  severity: `warning`  fixable: false

run `drift explain drift.performance.like-leading-wildcard` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "performance" rule, fires at `warning` by default.

## configuration

```toml
[rules]
"drift.performance.like-leading-wildcard" = "error"     # or warning, info, off
```


