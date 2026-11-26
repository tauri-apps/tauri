# Tauri Bench

Internal benchmarks of Tauri. They run on CI for every commit on `dev` (Linux, Windows and macOS) and the results are published to the [benchmark_results](https://github.com/tauri-apps/benchmark_results) repository.

## Layout

| Path                           | What it is                                                                                   |
| ------------------------------ | -------------------------------------------------------------------------------------------- |
| `src/run_benchmark.rs`         | The runner: runs the apps, measures them and writes `bench.json`                             |
| `src/build_benchmark_jsons.rs` | Appends `bench.json` to the history in the benchmark_results repository                      |
| `tests/common`                 | Tiny crate the apps use to report what they measured to the runner                           |
| `tests/helloworld`             | The smallest app: startup, idle usage, cost of a window, sizes, build times                  |
| `tests/assets`                 | Hello world with a 4 MB generated frontend: the asset protocol                               |
| `tests/ipc`                    | Commands, channels and events timed in a loop from the frontend                              |
| `tests/cpu_intensive`          | A web worker computing prime numbers: a calibration of the machine, not a benchmark of Tauri |
| `micro`                        | Core code paths that run without a webview, measured in instructions                         |

## What is measured

Each result is a `BenchResult` (see `src/utils.rs`). Timings taken on shared CI machines are noisy, so the fields are listed from the most to the least trustworthy.

| Field                           | Platforms | Meaning                                                                                                                                                                      |
| ------------------------------- | --------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `instructions`                  | Linux     | Instructions executed by each `micro` scenario, counted by callgrind. The same on every run                                                                                  |
| `binary_size`                   | all       | Size of each app, of the `wry` and `tauri` rlibs, of hello world without tauri's default features (`_minimal`) and with Cargo's default release profile (`_default_profile`) |
| `bundle_size`                   | all       | Size of the installers of hello world                                                                                                                                        |
| `cargo_deps`                    | all       | Number of crates `tauri` depends on, per target OS                                                                                                                           |
| `syscall_count`, `thread_count` | Linux     | Syscalls made and threads or processes created by a run of each app (`strace`)                                                                                               |
| `max_memory`                    | Linux     | Sum of the peak resident memory of the processes of each app                                                                                                                 |
| `metrics`                       | all       | What each app measured about itself, see below                                                                                                                               |
| `build_time`                    | all       | Seconds to build hello world: clean and incremental dev builds, clean release build                                                                                          |
| `exec_time`                     | all       | Seconds between spawning each app and it exiting. Mostly the webview booting                                                                                                 |
| `environment`                   | all       | Compiler, webview, CPU and runner image the results were measured with                                                                                                       |

`metrics` holds the median of several runs. Names ending in `_ms` are milliseconds and `_bytes` are bytes:

- `startup_*_ms`: time since the process was spawned until `main` ran, the window was created, the page started loading, the first IPC call arrived (`first_invoke`) and all assets were loaded (`tauri_assets`).
- `invoke_*`, `channel_*`, `event_*` (`tauri_ipc`): average time of one operation, measured with `performance.now()` over a loop inside the app.
- `asset_fetch_1mb_ms` (`tauri_assets`): average time to fetch a 1 MB asset.
- `memory_bytes`: memory used by the app and its webview processes once it is done (Linux, Windows).
- `idle_*` (`tauri_hello_world`): memory, CPU time and wakeups of the app doing nothing.
- `open_window_ms`, `window_memory_bytes` (`tauri_hello_world`): cost of each additional window.
- `compute_ms` (`tauri_cpu_intensive`): how fast the machine ran the calibration. Compare the other timings of a run against it before blaming a commit.

The apps are built in production mode (their `prod` feature, which enables `custom-protocol`), with the workspace's size optimized release profile.

## Running locally

Linux needs `strace`, and `valgrind` for `--instructions`. Results are written to `target/<triple>/release/bench.json`.

```sh
TARGET=x86_64-unknown-linux-gnu
cargo build --release --target $TARGET \
  -p bench_helloworld -p bench_cpu_intensive -p bench_ipc -p bench_assets \
  --features bench_helloworld/prod,bench_cpu_intensive/prod,bench_ipc/prod,bench_assets/prod
cargo run --manifest-path bench/Cargo.toml --bin run_benchmark
```

Three measurements build into `target/bench-build` and take minutes, so they are opt-in:

- `--build-time`: time the builds of hello world (removes `target/bench-build` first)
- `--bundle`: build the CLI and the installers of hello world
- `--instructions`: build `micro` and run it under callgrind

CI builds the apps with a pinned nightly, `build-std` and `panic=immediate-abort` (see `.github/workflows/bench.yml`), so sizes measured locally with a stable toolchain are larger.

## Adding a benchmark

- A new app: call `bench_common::init()` first thing in `main`, record values with `bench_common::mark` and `bench_common::set`, and end with `bench_common::finish()`. Add it to `get_all_benchmarks` in the runner and to the build step of the workflow.
- A new instruction count: add a scenario to `micro/src/main.rs`.
