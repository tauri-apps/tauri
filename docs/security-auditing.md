# Security auditing with `tauri audit`

This page documents the `tauri audit` CLI (MVP) which scans Rust and JavaScript dependencies for known vulnerabilities and produces machine-friendly outputs.

Usage (local):

- Run a human summary:

```
cargo tauri audit --format human
```

- Run JSON output (useful in CI):

```
cargo tauri audit --format json
```

- Write SARIF for GitHub code scanning (via config):

Add to your `tauri.conf.json`:

```json
{
  "build": {
    "audit": {
      "mode": "error",
      "failOn": "high",
      "rust": {
        "ignore": ["RUSTSEC-2024-0001"]
      },
      "js": {
        "ignore": ["left-pad"]
      },
      "output": {
        "formats": ["sarif", "json"],
        "sarifPath": "target/tauri-audit.sarif"
      }
    }
  }
}

GitHub Actions (upload SARIF):

```yaml
name: Upload SARIF
on: [push]
jobs:
  sarif:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Run Tauri audit
        run: |
          cargo tauri audit --format sarif
      - name: Upload SARIF to GitHub
        uses: github/codeql-action/upload-sarif@v2
        with:
          sarif_file: target/tauri-audit.sarif
```

Exit codes

| Code | Meaning |
|---:|:---|
| 0 | No policy violation (including `warn` mode) |
| 1 | Policy violation (mode=`error` and `failOn` matched) |
| 2 | Audit tool execution failure (tool missing OR command failed AND produced no parsable JSON) |
| 3 | Config/schema/argument error |

```

Notes:

- By default `tauri audit` will not auto-install external tools. Use `--install-tools` to opt-in for rust provider installation attempts.
- The current MVP supports `cargo-audit` for Rust and `npm/pnpm/yarn` audit JSON shapes for JS.
- Exit codes follow `build.audit.mode` semantics: `error` will exit non-zero when high/critical findings exist.

See the `tauri` CLI help for available flags.
