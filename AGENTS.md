<!-- crag:auto-start -->
# AGENTS.md

> Generated from governance.md by crag. Regenerate: `crag compile --target agents-md`

## Project: tauri-workspace


## Quality Gates

All changes must pass these checks before commit:

### Lint
1. `npm run format:check`
2. `cargo clippy -- -D warnings`
3. `cargo fmt --check`

### Test
1. `npm run test`
2. `cargo test`

### Build
1. `npm run build`

### Ci (inferred from workflow)
1. `pnpm build`
2. `cargo build --manifest-path ./crates/tauri-schema-generator/Cargo.toml`
3. `cargo build --manifest-path ./crates/tauri-cli/Cargo.toml`
4. `cargo test --test '*' -- --ignored`
5. `pnpm test`
6. `cargo fmt --all -- --check`
7. `pnpm eslint:check`
8. `cargo clippy --all-targets --all-features -- -D warnings`

### Contributor docs (advisory — confirm before enforcing)
1. `cargo clippy  # from .github/PULL_REQUEST_TEMPLATE.md`

## Coding Standards

- Stack: node, rust, typescript
- Conventional commits (feat:, fix:, docs:, etc.)
- Commit trailer: Co-Authored-By: Claude <noreply@anthropic.com>

## Architecture

- Type: monorepo (cargo)

## Key Directories

- `.github/` — CI/CD
- `crates/` — workspace crates
- `packages/` — workspace packages

## Testing

- Framework: cargo test
- Layout: flat

## Code Style

- Indent: 2 spaces
- Formatter: prettier

## Anti-Patterns

Do not:
- Do not leave `console.log` in production code — use a proper logger
- Do not use synchronous filesystem APIs in request handlers
- Do not use `unwrap()` in library code — return `Result` instead
- Do not `clone()` without justification — prefer borrowing
- Do not use `unsafe` without a safety comment explaining invariants
- Do not use `any` type — use `unknown` or proper types instead
- Do not use `@ts-ignore` — fix the type error or use `@ts-expect-error` with a reason
- Prefer `as const` over `enum` for string unions

## Security

- No hardcoded secrets — grep for sk_live, AKIA, password= before commit

## Workflow

1. Read `governance.md` at the start of every session — it is the single source of truth.
2. Run all mandatory quality gates before committing.
3. If a gate fails, fix the issue and re-run only the failed gate.
4. Use the project commit conventions for all changes.

<!-- crag:auto-end -->
