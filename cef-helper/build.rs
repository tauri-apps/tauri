// CEF executes this helper as a bare subprocess, so it resolves its libcef
// dependency through its own dynamic-loader search path — not the browser
// process's preloaded copy. The helper is always distributed next to libcef
// (the cargo target dir and a bundle's resource dir alike), so an `$ORIGIN`
// RUNPATH points the loader there. Mirrors what tauri-ffi's build script does
// for the cdylib.
//
// Set here rather than through the CLI appending to RUSTFLAGS: cargo ignores
// RUSTFLAGS entirely when CARGO_ENCODED_RUSTFLAGS (or a `.cargo/config` rustflag)
// is present, so the env approach could silently drop the rpath. A build script
// link-arg composes with all of those and applies however the crate is built.
fn main() {
  if std::env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("linux") {
    println!("cargo:rustc-link-arg-bins=-Wl,-rpath,$ORIGIN");
  }
}
