---
"cli.rs": patch
---

The CLI will not automatically run `strip` on release binaries anymore. Use the [`strip`] profile setting stabilized with Cargo 1.59.

[`strip`]: https://doc.rust-lang.org/cargo/reference/profiles.html#strip
