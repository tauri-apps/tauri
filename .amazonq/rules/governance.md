<!-- crag:auto-start -->
# Amazon Q Rules — tauri-workspace

> Generated from governance.md by crag. Regenerate: `crag compile --target amazonq`

## About

(No description)

**Stack:** node, rust, typescript

**Runtimes detected:** node, rust

## How Amazon Q Should Behave on This Project

### Code Generation

1. **Run governance gates before suggesting commits.** The gates below define the quality bar.
2. **Respect classifications:** MANDATORY (default) blocks on failure; OPTIONAL warns; ADVISORY is informational only.
3. **Respect scopes:** Path-scoped gates run from that directory. Conditional gates skip when their file does not exist.
4. **No secrets.** - No hardcoded secrets — grep for sk_live, AKIA, password= before commit
5. **Minimal diffs.** Prefer editing existing code over creating new files. Do not refactor unrelated areas.

### Quality Gates

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

### Commit Style

Use conventional commits: `feat(scope): description`, `fix(scope): description`, `docs: description`, etc.
Commit trailer: Co-Authored-By: Claude <noreply@anthropic.com>

### Boundaries

- All file operations must stay within this repository.
- No destructive shell commands (rm -rf above repo root, DROP TABLE without confirmation, force-push to main).
- No new dependencies without an explicit reason.

## Authoritative Source

When these instructions seem to conflict with something in the repo, **`.claude/governance.md` is the source of truth**. This file is a compiled view.

---

**Tool:** crag — https://www.npmjs.com/package/@whitehatd/crag

<!-- crag:auto-end -->
