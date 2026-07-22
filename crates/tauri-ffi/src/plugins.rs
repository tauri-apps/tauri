// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Host-defined plugins. A plugin bundles an optional JavaScript init script
//! (injected into every webview) and a set of commands invoked from the
//! frontend as `plugin:<name>|<command>`. Plugins are built up through opaque
//! handles, then consumed by [`tauri_app_builder_add_plugin`], which moves
//! them into the [`crate::state::BuilderState`]. The heavy lifting — attaching
//! a `tauri::plugin::Builder` and synthesizing the ACL manifest — happens at
//! [`crate::build_app`] time.

use std::os::raw::c_char;

use crate::error::{catch, fail, ERR_INVALID_ARG, ERR_INVALID_HANDLE, OK};
use crate::state::{self, Entry, PluginState};
use crate::try_cstr;

/// Creates a plugin definition. `core` and `tauri` are reserved names; using
/// one fails later at `tauri_app_build`.
#[no_mangle]
pub unsafe extern "C" fn tauri_plugin_new(name: *const c_char, out_plugin: *mut u64) -> i32 {
  catch(|| {
    let name = try_cstr!(name);
    if out_plugin.is_null() {
      return fail(ERR_INVALID_ARG, "out_plugin is null");
    }
    if name.is_empty() {
      return fail(ERR_INVALID_ARG, "plugin name is empty");
    }
    let id = state::insert(Entry::Plugin(PluginState {
      name: name.to_string(),
      init_script: None,
      commands: Default::default(),
    }));
    unsafe { *out_plugin = id };
    OK
  })
}

/// Sets JavaScript injected into every webview before page scripts run.
#[no_mangle]
pub unsafe extern "C" fn tauri_plugin_set_init_script(plugin: u64, js: *const c_char) -> i32 {
  catch(|| {
    let js = try_cstr!(js);
    state::with_plugin(plugin, |p| {
      p.init_script = Some(js.to_string());
      OK
    })
  })
}

/// Registers a command handled by this plugin.
#[no_mangle]
pub unsafe extern "C" fn tauri_plugin_register_command(plugin: u64, name: *const c_char) -> i32 {
  catch(|| {
    let name = try_cstr!(name);
    state::with_plugin(plugin, |p| {
      p.commands.insert(name.to_string());
      OK
    })
  })
}

// ---------------------------------------------------------------------------
// compiled-in plugins that need host arguments
//
// Every other first-party plugin is registered unconditionally by
// [`crate::build_app`]; these two are opt-in because they take arguments and
// change process-wide behavior (see the comment there).

/// Serves the app frontend over `http://localhost:<port>` instead of the custom
/// protocol, for webviews that need a real HTTP origin. The host still points
/// its windows at that URL (`app.windows[].url` in the config).
#[no_mangle]
pub extern "C" fn tauri_app_builder_enable_localhost(builder: u64, port: u32) -> i32 {
  catch(|| {
    let Ok(port) = u16::try_from(port) else {
      return fail(ERR_INVALID_ARG, format!("invalid port {port}"));
    };
    state::with_builder(builder, |b| {
      b.localhost_port = Some(port);
      OK
    })
  })
}

/// Makes the process refuse to launch a second instance: a later launch hands
/// its command line to the running app and exits, and the running app receives
/// `{"type":"single-instance","args":[...],"cwd":...}` on the event queue.
#[no_mangle]
pub extern "C" fn tauri_app_builder_enable_single_instance(builder: u64) -> i32 {
  catch(|| {
    state::with_builder(builder, |b| {
      b.single_instance = true;
      OK
    })
  })
}

/// Attaches a plugin to the builder, consuming the plugin handle.
#[no_mangle]
pub extern "C" fn tauri_app_builder_add_plugin(builder: u64, plugin: u64) -> i32 {
  catch(|| {
    let Some(Entry::Plugin(plugin_state)) = state::remove(plugin) else {
      return fail(ERR_INVALID_HANDLE, "invalid plugin handle");
    };
    // Re-insert the plugin on any builder error so the handle isn't lost.
    let mut taken = Some(plugin_state);
    let code = state::with_builder(builder, |b| {
      b.plugins.push(taken.take().unwrap());
      OK
    });
    if let Some(plugin_state) = taken {
      state::insert(Entry::Plugin(plugin_state));
    }
    code
  })
}
