use cef::{args::Args, *};

mod notification;

/// Whether the CEF helper subprocess should emit its diagnostic `[cef-helper-*]`
/// traces. Off by default: CEF spawns a fresh helper for every renderer / GPU /
/// utility process, so unconditional prints flood stderr on every launch. Set
/// `OPENHUMAN_CEF_HELPER_VERBOSE=1` in the environment to re-enable them while
/// debugging (env propagates to the spawned subprocesses).
pub(crate) fn verbose() -> bool {
  std::env::var_os("OPENHUMAN_CEF_HELPER_VERBOSE").is_some()
}

fn main() {
  if verbose() {
    eprintln!("[cef-helper-main] starting helper main");
  }
  let args = Args::new();

  #[cfg(all(target_os = "macos", feature = "sandbox"))]
  let _sandbox = {
    let mut sandbox = cef::sandbox::Sandbox::new();
    sandbox.initialize(args.as_main_args());
    sandbox
  };

  #[cfg(target_os = "macos")]
  let _loader = {
    let loader = library_loader::LibraryLoader::new(&std::env::current_exe().unwrap(), true);
    assert!(loader.load());
    loader
  };

  // Populate the API version hash for this process before constructing any
  // CEF C struct. Without this, `cef_app_t.version` is left at -1 and the
  // browser process aborts with `CefApp_0_CToCpp called with invalid version
  // -1` on every subprocess spawn. The browser process does this in
  // `tauri-runtime-cef::CefRuntime::init` — the helper must do it too.
  let _ = cef::api_hash(cef::sys::CEF_API_VERSION_LAST, 0);

  let mut app = notification::NotifyApp::new();
  if verbose() {
    eprintln!("[cef-helper-main] executing subprocess with NotifyApp");
  }
  execute_process(
    Some(args.as_main_args()),
    Some(&mut app),
    std::ptr::null_mut(),
  );
}
