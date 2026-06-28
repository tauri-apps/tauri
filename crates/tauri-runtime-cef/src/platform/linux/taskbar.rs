// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use dlopen2::wrapper::{Container, WrapperApi};
use std::{cell::RefCell, ffi::CString, os::raw::c_char};
use tauri_runtime::{ProgressBarState, ProgressBarStatus};

pub(super) fn set_progress_bar(state: ProgressBarState) {
  TASKBAR_INDICATOR.with_borrow_mut(|taskbar| taskbar.update(state));
}

pub(super) fn set_badge_count(count: Option<i64>, desktop_filename: Option<String>) {
  TASKBAR_INDICATOR.with_borrow_mut(|taskbar| taskbar.update_count(count, desktop_filename));
}

#[derive(WrapperApi)]
struct UnityLib {
  unity_launcher_entry_get_for_desktop_id: unsafe extern "C" fn(id: *const c_char) -> *const isize,
  unity_inspector_get_default: unsafe extern "C" fn() -> *const isize,
  unity_inspector_get_unity_running: unsafe extern "C" fn(inspector: *const isize) -> i32,
  unity_launcher_entry_set_progress: unsafe extern "C" fn(entry: *const isize, value: f64) -> i32,
  unity_launcher_entry_set_progress_visible:
    unsafe extern "C" fn(entry: *const isize, value: i32) -> i32,
  unity_launcher_entry_set_count: unsafe extern "C" fn(entry: *const isize, value: i64) -> i32,
  unity_launcher_entry_set_count_visible:
    unsafe extern "C" fn(entry: *const isize, value: bool) -> bool,
}

struct TaskbarIndicator {
  desktop_filename: Option<String>,
  desktop_filename_c_str: Option<CString>,

  unity_lib: Option<Container<UnityLib>>,
  attempted_load: bool,

  unity_inspector: Option<*const isize>,
  unity_entry: Option<*const isize>,
}

thread_local! {
  static TASKBAR_INDICATOR: RefCell<TaskbarIndicator> = RefCell::new(TaskbarIndicator::new());
}

impl TaskbarIndicator {
  fn new() -> Self {
    Self {
      desktop_filename: None,
      desktop_filename_c_str: None,

      unity_lib: None,
      attempted_load: false,

      unity_inspector: None,
      unity_entry: None,
    }
  }

  fn ensure_lib_load(&mut self) {
    if self.attempted_load {
      return;
    }

    self.attempted_load = true;

    self.unity_lib = unsafe {
      Container::load("libunity.so.4")
        .or_else(|_| Container::load("libunity.so.6"))
        .or_else(|_| Container::load("libunity.so.9"))
        .ok()
    };

    if let Some(unity_lib) = &self.unity_lib {
      let handle = unsafe { unity_lib.unity_inspector_get_default() };
      if !handle.is_null() {
        self.unity_inspector = Some(handle);
      }
    }
  }

  fn ensure_entry_load(&mut self) {
    if let Some(unity_lib) = &self.unity_lib
      && let Some(id) = &self.desktop_filename_c_str
    {
      let handle = unsafe { unity_lib.unity_launcher_entry_get_for_desktop_id(id.as_ptr()) };
      if !handle.is_null() {
        self.unity_entry = Some(handle);
      }
    }
  }

  fn is_unity_running(&self) -> bool {
    if let Some(inspector) = self.unity_inspector
      && let Some(unity_lib) = &self.unity_lib
    {
      return unsafe { unity_lib.unity_inspector_get_unity_running(inspector) } == 1;
    }

    false
  }

  fn update(&mut self, progress: ProgressBarState) {
    if let Some(uri) = progress.desktop_filename {
      self.desktop_filename = Some(uri);
      self.desktop_filename_c_str = None;
      self.unity_entry = None;
    }

    self.ensure_lib_load();

    if !self.is_unity_running() {
      return;
    }

    if self.desktop_filename_c_str.is_none()
      && let Some(uri) = &self.desktop_filename
    {
      self.desktop_filename_c_str = Some(CString::new(uri.as_str()).unwrap_or_default());
    }

    if self.unity_entry.is_none() {
      self.ensure_entry_load();
    }

    if let Some(unity_lib) = &self.unity_lib
      && let Some(unity_entry) = self.unity_entry
    {
      if let Some(progress) = progress.progress {
        let progress = progress.min(100) as f64 / 100.0;
        unsafe { unity_lib.unity_launcher_entry_set_progress(unity_entry, progress) };
      }

      if let Some(status) = progress.status {
        let is_visible = !matches!(status, ProgressBarStatus::None);
        unsafe {
          unity_lib
            .unity_launcher_entry_set_progress_visible(unity_entry, if is_visible { 1 } else { 0 })
        };
      }
    }
  }

  fn update_count(&mut self, count: Option<i64>, desktop_filename: Option<String>) {
    if let Some(uri) = desktop_filename {
      self.desktop_filename = Some(uri);
      self.desktop_filename_c_str = None;
      self.unity_entry = None;
    }

    self.ensure_lib_load();

    if !self.is_unity_running() {
      return;
    }

    if self.desktop_filename_c_str.is_none()
      && let Some(uri) = &self.desktop_filename
    {
      self.desktop_filename_c_str = Some(CString::new(uri.as_str()).unwrap_or_default());
    }

    if self.unity_entry.is_none() {
      self.ensure_entry_load();
    }

    if let Some(unity_lib) = &self.unity_lib
      && let Some(unity_entry) = self.unity_entry
    {
      if let Some(count) = count {
        unsafe { unity_lib.unity_launcher_entry_set_count(unity_entry, count) };
        unsafe { unity_lib.unity_launcher_entry_set_count_visible(unity_entry, true) };
      } else {
        unsafe { unity_lib.unity_launcher_entry_set_count(unity_entry, 0) };
        unsafe { unity_lib.unity_launcher_entry_set_count_visible(unity_entry, false) };
      }
    }
  }
}
