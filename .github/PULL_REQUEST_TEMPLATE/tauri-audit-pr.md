<!-- Describe the change and why it matters -->

### Summary

Add `tauri audit` MVP: cargo-audit based Rust provider, JS audit parsing, SARIF output, and `build.audit` config schema.

### Implementation

- `crates/tauri-cli/src/audit.rs`: CLI command, providers, parsers, SARIF output.
- `crates/tauri-cli/config.schema.json`: new `BuildAuditConfig` schema and `build.audit` property.
- `docs/security-auditing.md`: usage docs.
- Unit tests for parsing helpers in `audit.rs`.

### Testing

- `cargo test -p tauri-cli` should pass for new unit tests.
- Manual: run `cargo tauri audit --format human` in a repo with lockfiles.

### Follow-ups

- Add `build.audit` schema descriptions in website docs.
- Add SARIF mapping refinements and GitHub Code Scanning integration.
- Add JS provider auto-installation option or guided instructions.

