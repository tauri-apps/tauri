---
trigger: always_on
description: Governance rules for tauri-workspace — compiled from governance.md by crag
---

# Windsurf Rules — tauri-workspace

Generated from governance.md by crag. Regenerate: `crag compile --target windsurf`

## Project

(No description)

**Stack:** node, rust, typescript

## Runtimes

node, rust

## Cascade Behavior

When Windsurf's Cascade agent operates on this project:

- **Always read governance.md first.** It is the single source of truth for quality gates and policies.
- **Run all mandatory gates before proposing changes.** Stop on first failure.
- **Respect classifications.** OPTIONAL gates warn but don't block. ADVISORY gates are informational.
- **Respect path scopes.** Gates with a `path:` annotation must run from that directory.
- **No destructive commands.** Never run rm -rf, dd, DROP TABLE, force-push to main, curl|bash, docker system prune.
- - No hardcoded secrets — grep for sk_live, AKIA, password= before commit
- **Conventional commits.** Every commit must follow `<type>(<scope>): <description>`.
- **Commit trailer:** Co-Authored-By: Claude <noreply@anthropic.com>

## Quality Gates (run in order)

1. `npm run format:check`
2. `cargo clippy -- -D warnings`
3. `cargo fmt --check`
4. `npm run test`
5. `cargo test`
6. `npm run build`
7. `pnpm build`
8. `cargo build --manifest-path ./crates/tauri-schema-generator/Cargo.toml`
9. `cargo build --manifest-path ./crates/tauri-cli/Cargo.toml`
10. `cargo test --test '*' -- --ignored`
11. `pnpm test`
12. `cargo fmt --all -- --check`
13. `pnpm eslint:check`
14. `cargo clippy --all-targets --all-features -- -D warnings`
15. `cargo clippy  # from .github/PULL_REQUEST_TEMPLATE.md`

## Rules of Engagement

1. **Minimal changes.** Don't rewrite files that weren't asked to change.
2. **No new dependencies** without explicit approval.
3. **Prefer editing** existing files over creating new ones.
4. **Always explain** non-obvious changes in commit messages.
5. **Ask before** destructive operations (delete, rename, migrate schema).

---

**Tool:** crag — https://www.npmjs.com/package/@whitehatd/crag
