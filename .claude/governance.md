# Governance — tauri-workspace
# Inferred by crag analyze — review and adjust as needed

## Identity
- Project: tauri-workspace
- Stack: node, rust, typescript
- Workspace: pnpm

## Gates (run in order, stop on failure)
### Lint
- npm run format:check
- cargo clippy -- -D warnings
- cargo fmt --check

### Test
- npm run test
- cargo test

### Build
- npm run build

### CI (inferred from workflow)
- pnpm build
- cargo build --manifest-path ./crates/tauri-schema-generator/Cargo.toml
- cargo build --manifest-path ./crates/tauri-cli/Cargo.toml
- cargo test --test '*' -- --ignored
- pnpm test
- cargo fmt --all -- --check
- pnpm eslint:check
- cargo clippy --all-targets --all-features -- -D warnings

### Contributor docs (ADVISORY — confirm before enforcing)
- cargo clippy  # from .github/PULL_REQUEST_TEMPLATE.md

## Advisories (informational, not enforced)
- actionlint  # [ADVISORY]

## Branch Strategy
- Trunk-based development
- Conventional commits
- Commit trailer: Co-Authored-By: Claude <noreply@anthropic.com>

## Security
- No hardcoded secrets — grep for sk_live, AKIA, password= before commit

## Autonomy
- Auto-commit after gates pass

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

## Dependencies
- Package manager: pnpm (pnpm-lock.yaml)
- Rust: >=1.77.2
- Rust-edition: 2021

## Import Conventions
- Module system: CJS

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

