# drift for vscode

SQL linter and formatter, powered by drift. Diagnostics on save and on type, code actions for the quick-fixable rules, format on save.

## install

1. Install the drift binary:
   ```sh
   brew install f4rkh4d/tap/drift
   # or
   cargo install drift-sql
   # or
   curl -fsSL https://drift.frkhd.com/install.sh | sh
   ```
2. Install this extension from the marketplace.
3. Open a `.sql` file. drift starts in the background and diagnostics appear inline.

## features

- 80+ rules across style, correctness, performance, security, portability, conventions, ambiguity
- 5 dialects: postgres (primary), mysql, sqlite, bigquery, snowflake
- Code actions for the quick-fixable rules (keyword case, trailing whitespace, etc.)
- Honors your repo's `drift.toml` for rule severity overrides
- Honors `.drift-baseline.json` if present (suppresses legacy violations)

## settings

| setting | default | what |
|---|---|---|
| `drift.path` | `drift` | Path to the drift binary. Override if drift is not on `$PATH`. |
| `drift.dialect` | `postgres` | Dialect override. Per-file detection still applies. |
| `drift.trace.server` | `off` | LSP trace level for the drift output panel. |

## development

```sh
npm install
npm run build
# F5 to launch an extension host with this in dev mode
```

`npm run package` produces a `.vsix` you can install locally with `code --install-extension drift-sql-*.vsix`.

## license

MIT.
