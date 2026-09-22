# Windows runtime-handle clone/drop reproducer

`runtime_handle_clone.rs` isolates the handle-cloning path discussed in #15408
and #15411. It creates one Wry runtime, synchronizes eight workers, and attempts
100,000 clone/drop pairs per worker. The runtime remains alive until all workers
join. It uses public safe APIs, with no WebView, custom protocol, Debug formatting,
or patch-specific post-shutdown assertions.

Run on a Windows desktop, using a fresh process for each trial:

```sh
cargo run -p tauri-runtime-wry --example runtime_handle_clone
```

A successful run prints `JOINED workers` and
`PASS clone/drop and runtime teardown`. An affected runtime may abort or terminate
with heap corruption. Save stderr and the exit status. This is a manual
reproducer, not a test that can abort the shared test harness. A passing run
alone is not proof that a data race is absent.

## Reproducing against published crates

Copy the example to `src/main.rs` in a standalone project with this Cargo.toml:

```toml
[package]
name = "runtime-handle-clone-repro"
version = "0.1.0"
edition = "2021"

[dependencies]
tauri-runtime = "=2.11.3"
tauri-runtime-wry = "=2.11.4"
tao = "=0.35.3"
```

Run `cargo run`, then repeat the executable in fresh processes. For comparisons,
retain Cargo.lock and change only the runtime-wry dependency source.

Observed on Windows x86_64 with rustc 1.98.1: the original published 2.11.4 crate
failed in 5/5 debug runs with `0xC0000374` (heap corruption). Captured stderr also
reported an `alloc::rc` unsafe-precondition violation on a worker before JOINED.
A local owner-thread-storage candidate passed identical source and dependency
versions in 5/5 runs. That candidate is distinct from PR #15411; these results
do not validate that PR. Release A/B and other platforms are untested.

The same reproducer also failed in 3/3 debug runs on upstream `dev` commit
`0349b6fb8a77739146d6e0071cad33d9c083fc4f` (Tao 0.37.0, Wry 0.57.0), again with
`0xC0000374` and a worker-thread `alloc::rc` unsafe-precondition violation.
