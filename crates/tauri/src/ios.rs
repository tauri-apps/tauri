// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use swift_rs::{swift, SRString, SwiftArg};

use std::{
  ffi::c_void,
  os::raw::{c_char, c_int, c_ulonglong},
};

type PluginMessageCallbackFn = unsafe extern "C" fn(c_int, c_int, *const c_char);
pub struct PluginMessageCallback(pub PluginMessageCallbackFn);

impl<'a> SwiftArg<'a> for PluginMessageCallback {
  type ArgType = PluginMessageCallbackFn;

  unsafe fn as_arg(&'a self) -> Self::ArgType {
    self.0
  }
}

type ChannelSendDataCallbackFn = unsafe extern "C" fn(c_ulonglong, *const c_char);
pub struct ChannelSendDataCallback(pub ChannelSendDataCallbackFn);

impl<'a> SwiftArg<'a> for ChannelSendDataCallback {
  type ArgType = ChannelSendDataCallbackFn;

  unsafe fn as_arg(&'a self) -> Self::ArgType {
    self.0
  }
}

swift!(pub fn run_plugin_command(
  id: i32,
  name: &SRString,
  method: &SRString,
  data: &SRString,
  callback: PluginMessageCallback,
  send_channel_data_callback: ChannelSendDataCallback
));
swift!(pub fn register_plugin(
  name: &SRString,
  plugin: *const c_void,
  config: &SRString,
  webview: *const c_void
));
swift!(pub fn on_webview_created(webview: *const c_void, controller: *const c_void));
