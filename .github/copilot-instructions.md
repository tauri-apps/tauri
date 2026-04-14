<!-- crag:auto-start -->
# Copilot Instructions — tauri-workspace

> Generated from governance.md by crag. Regenerate: `crag compile --target copilot`



**Stack:** node, rust, typescript

## Runtimes

node, rust

## Quality Gates

When you propose changes, the following checks must pass before commit:

- **lint**: `npm run format:check`
- **lint**: `cargo clippy -- -D warnings`
- **lint**: `cargo fmt --check`
- **test**: `npm run test`
- **test**: `cargo test`
- **build**: `npm run build`
- **ci (inferred from workflow)**: `pnpm build`
- **ci (inferred from workflow)**: `cargo build --manifest-path ./crates/tauri-schema-generator/Cargo.toml`
- **ci (inferred from workflow)**: `cargo build --manifest-path ./crates/tauri-cli/Cargo.toml`
- **ci (inferred from workflow)**: `cargo test --test '*' -- --ignored`
- **ci (inferred from workflow)**: `pnpm test`
- **ci (inferred from workflow)**: `cargo fmt --all -- --check`
- **ci (inferred from workflow)**: `pnpm eslint:check`
- **ci (inferred from workflow)**: `cargo clippy --all-targets --all-features -- -D warnings`
- **contributor docs (advisory — confirm before enforcing)**: `cargo clippy  # from .github/PULL_REQUEST_TEMPLATE.md`

## Expectations for AI-Assisted Code

1. **Run gates before suggesting a commit.** If you cannot run them (no shell access), explicitly remind the human to run them.
2. **Respect classifications.** `MANDATORY` gates must pass. `OPTIONAL` gates should pass but may be overridden with a note. `ADVISORY` gates are informational only.
3. **Respect workspace paths.** When a gate is scoped to a subdirectory, run it from that directory.
4. **No hardcoded secrets.** - No hardcoded secrets — grep for sk_live, AKIA, password= before commit
5. **Conventional commits** for all changes. Trailer: `Co-Authored-By: Claude <noreply@anthropic.com>`
6. **Conservative changes.** Do not rewrite unrelated files. Do not add new dependencies without explaining why.

## Tool Context

This project uses **crag** (https://www.npmjs.com/package/@whitehatd/crag) as its AI-agent governance layer. The `governance.md` file is the authoritative source. If you have shell access, run `crag check` to verify the infrastructure and `crag diff` to detect drift.

<!-- crag:auto-end -->
