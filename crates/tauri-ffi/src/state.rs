// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Handle registry. Objects crossing the FFI boundary are opaque `u64` ids
//! into a global map, so stale ids fail with `ERR_INVALID_HANDLE` instead of
//! dangling-pointer UB — callers are typically GC'd languages.

use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{mpsc, Arc, Mutex, OnceLock};

use tauri::ipc::InvokeResolver;
use tauri::utils::config::Config;
use tauri::Wry;

/// Pre-build app state, mutated by `tauri_app_builder_*`.
pub struct BuilderState {
  pub config: Config,
  pub assets_dir: Option<PathBuf>,
  pub commands: HashSet<String>,
  pub capabilities: Vec<String>,
  pub plugins: Vec<PluginState>,
}

/// A host-defined plugin, mutated by `tauri_plugin_*` and moved into a
/// `BuilderState` by `tauri_app_builder_add_plugin`.
pub struct PluginState {
  pub name: String,
  pub init_script: Option<String>,
  pub commands: HashSet<String>,
}

/// Post-build app state. Everything here is `Send + Sync` and callable from
/// any thread while the event loop occupies the main thread.
pub struct AppState {
  pub handle: tauri::AppHandle<Wry>,
  /// Serialized event stream consumed by `tauri_events_next`.
  pub rx: Mutex<mpsc::Receiver<String>>,
  pub tx: mpsc::Sender<String>,
}

pub enum Entry {
  Builder(BuilderState),
  Plugin(PluginState),
  App(Arc<AppState>),
  Window(tauri::WebviewWindow<Wry>),
  Resolver(InvokeResolver<Wry>),
}

static NEXT_ID: AtomicU64 = AtomicU64::new(1);

fn registry() -> &'static Mutex<HashMap<u64, Entry>> {
  static REGISTRY: OnceLock<Mutex<HashMap<u64, Entry>>> = OnceLock::new();
  REGISTRY.get_or_init(Default::default)
}

pub fn insert(entry: Entry) -> u64 {
  let id = NEXT_ID.fetch_add(1, Ordering::Relaxed);
  registry().lock().unwrap().insert(id, entry);
  id
}

pub fn remove(id: u64) -> Option<Entry> {
  registry().lock().unwrap().remove(&id)
}

pub fn app(id: u64) -> Option<Arc<AppState>> {
  match registry().lock().unwrap().get(&id) {
    Some(Entry::App(state)) => Some(Arc::clone(state)),
    _ => None,
  }
}

pub fn window(id: u64) -> Option<tauri::WebviewWindow<Wry>> {
  match registry().lock().unwrap().get(&id) {
    Some(Entry::Window(window)) => Some(window.clone()),
    _ => None,
  }
}

pub fn with_builder(id: u64, f: impl FnOnce(&mut BuilderState) -> i32) -> i32 {
  match registry().lock().unwrap().get_mut(&id) {
    Some(Entry::Builder(builder)) => f(builder),
    _ => crate::error::fail(crate::error::ERR_INVALID_HANDLE, "invalid builder handle"),
  }
}

pub fn with_plugin(id: u64, f: impl FnOnce(&mut PluginState) -> i32) -> i32 {
  match registry().lock().unwrap().get_mut(&id) {
    Some(Entry::Plugin(plugin)) => f(plugin),
    _ => crate::error::fail(crate::error::ERR_INVALID_HANDLE, "invalid plugin handle"),
  }
}

thread_local! {
  /// `tauri::App` owns the event loop and is `!Send`: it never leaves the
  /// thread that built it. `tauri_app_run` must therefore be called from the
  /// same thread as `tauri_app_build` (the process main thread on macOS).
  pub static LOCAL_APPS: RefCell<HashMap<u64, tauri::App<Wry>>> = RefCell::new(HashMap::new());
}
