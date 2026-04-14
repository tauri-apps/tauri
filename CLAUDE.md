<!-- crag:auto-start -->
# CLAUDE.md — tauri-workspace

> Generated from governance.md by crag. Regenerate: `crag compile --target claude`



**Stack:** node, rust, typescript
**Runtimes:** node, rust

## Quality Gates

Run these in order before committing. Stop on first MANDATORY failure:

- `npm run format:check`
- `cargo clippy -- -D warnings`
- `cargo fmt --check`
- `npm run test`
- `cargo test`
- `npm run build`
- `pnpm build`
- `cargo build --manifest-path ./crates/tauri-schema-generator/Cargo.toml`
- `cargo build --manifest-path ./crates/tauri-cli/Cargo.toml`
- `cargo test --test '*' -- --ignored`
- `pnpm test`
- `cargo fmt --all -- --check`
- `pnpm eslint:check`
- `cargo clippy --all-targets --all-features -- -D warnings`
- `cargo clippy  # from .github/PULL_REQUEST_TEMPLATE.md`

## Rules

1. Read `governance.md` at the start of every session — it is the single source of truth.
2. Run all mandatory quality gates before committing.
3. If a gate fails, attempt an automatic fix (lint/format) with bounded retry (max 2 attempts). If it still fails, escalate to the user.
4. Never modify files outside this repository.
5. Never run destructive system commands (`rm -rf /`, `DROP TABLE`, force-push to main).
- Use conventional commits (feat:, fix:, docs:, etc.)
- Commit trailer: `Co-Authored-By: Claude <noreply@anthropic.com>`

## Security

- No hardcoded secrets — grep for sk_live, AKIA, password= before commit

## Tool Context

This project uses **crag** (https://www.npmjs.com/package/@whitehatd/crag) as its governance engine. The `governance.md` file is the authoritative source. Run `crag audit` to detect drift and `crag compile --target all` to recompile all targets.

<!-- crag:auto-end -->
