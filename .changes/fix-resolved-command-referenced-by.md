---
'tauri-utils': 'patch:bug'
---

Make `ResolvedCommand::referenced_by` (and `ResolvedCommandReference`) unconditional instead of gating them behind `#[cfg(debug_assertions)]`. The `ToTokens` output of `generate_context!` varied with the proc-macro host's `debug-assertions` rather than the consumer's, so under mismatched profiles (e.g. `profile.release.build-override.debug-assertions = true`) the emitted struct literal referenced a field the consumer could not see, failing to compile with `error[E0560]: struct ... has no field named referenced_by`.
