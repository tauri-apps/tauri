<!-- crag:auto-start -->
# GEMINI.md

> Generated from governance.md by crag. Regenerate: `crag compile --target gemini`

## Project Context

- **Name:** tauri-workspace
- **Stack:** node, rust, typescript
- **Runtimes:** node, rust

## Rules

### Quality Gates

Run these checks in order before committing any changes:

1. [lint] `npm run format:check`
2. [lint] `cargo clippy -- -D warnings`
3. [lint] `cargo fmt --check`
4. [test] `npm run test`
5. [test] `cargo test`
6. [build] `npm run build`
7. [ci (inferred from workflow)] `pnpm build`
8. [ci (inferred from workflow)] `cargo build --manifest-path ./crates/tauri-schema-generator/Cargo.toml`
9. [ci (inferred from workflow)] `cargo build --manifest-path ./crates/tauri-cli/Cargo.toml`
10. [ci (inferred from workflow)] `cargo test --test '*' -- --ignored`
11. [ci (inferred from workflow)] `pnpm test`
12. [ci (inferred from workflow)] `cargo fmt --all -- --check`
13. [ci (inferred from workflow)] `pnpm eslint:check`
14. [ci (inferred from workflow)] `cargo clippy --all-targets --all-features -- -D warnings`
15. [contributor docs (advisory — confirm before enforcing)] `cargo clippy  # from .github/PULL_REQUEST_TEMPLATE.md`

### Security

- No hardcoded secrets — grep for sk_live, AKIA, password= before commit

### Workflow

- Conventional commits (feat:, fix:, docs:, chore:, etc.)
- Commit trailer: Co-Authored-By: Claude <noreply@anthropic.com>
- Run quality gates before committing
- Review security implications of all changes

<!-- crag:auto-end -->
