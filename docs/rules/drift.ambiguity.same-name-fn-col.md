# drift.ambiguity.same-name-fn-col

**function name collides with column name**  —  category: `ambiguity`  severity: `info`  fixable: false

run `drift explain drift.ambiguity.same-name-fn-col` for the interactive version.

## why

see the rule description in `drift explain`. short version: this is a "ambiguity" rule, fires at `info` by default.

## configuration

```toml
[rules]
"drift.ambiguity.same-name-fn-col" = "error"     # or warning, info, off
```


