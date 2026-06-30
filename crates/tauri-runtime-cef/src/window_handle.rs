// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#[cfg(windows)]
use raw_window_handle::{HasDisplayHandle, HasWindowHandle, RawDisplayHandle, RawWindowHandle};
#[cfg(windows)]
use winit::window::Window as WinitWindow;

pub(crate) struct SendRawWindowHandle(pub raw_window_handle::RawWindowHandle);
unsafe impl Send for SendRawWindowHandle {}

pub(crate) struct SendRawDisplayHandle(pub raw_window_handle::RawDisplayHandle);
unsafe impl Send for SendRawDisplayHandle {}

#[cfg(windows)]
#[derive(Clone, Copy, Debug)]
pub(crate) struct SoftbufferWindowHandle {
  display: RawDisplayHandle,
  window: RawWindowHandle,
}

#[cfg(windows)]
impl SoftbufferWindowHandle {
  pub(crate) fn new(window: &dyn WinitWindow) -> Option<Self> {
    Some(Self {
      display: window.display_handle().ok()?.as_raw(),
      window: window.window_handle().ok()?.as_raw(),
    })
  }
}

#[cfg(windows)]
impl HasDisplayHandle for SoftbufferWindowHandle {
  fn display_handle(
    &self,
  ) -> std::result::Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
    Ok(unsafe { raw_window_handle::DisplayHandle::borrow_raw(self.display) })
  }
}

#[cfg(windows)]
impl HasWindowHandle for SoftbufferWindowHandle {
  fn window_handle(
    &self,
  ) -> std::result::Result<raw_window_handle::WindowHandle<'_>, raw_window_handle::HandleError> {
    Ok(unsafe { raw_window_handle::WindowHandle::borrow_raw(self.window) })
  }
}
