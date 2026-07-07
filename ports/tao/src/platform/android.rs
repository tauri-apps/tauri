// Copyright 2014-2021 The winit contributors
// Copyright 2021-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0

#![cfg(target_os = "android")]

pub mod prelude {
  pub use crate::platform_impl::ndk_glue::*;
  pub use tao_macros::{android_fn, generate_package_name};
}
use crate::{
  event_loop::{EventLoop, EventLoopWindowTarget},
  platform_impl::ndk_glue::Rect,
  window::{Window, WindowBuilder},
};
use ndk::configuration::Configuration;

/// Additional methods on `EventLoop` that are specific to Android.
pub trait EventLoopExtAndroid {}

impl<T> EventLoopExtAndroid for EventLoop<T> {}

/// Additional methods on `EventLoopWindowTarget` that are specific to Android.
pub trait EventLoopWindowTargetExtAndroid {}

/// Additional methods on `Window` that are specific to Android.
pub trait WindowExtAndroid {
  fn content_rect(&self) -> Rect;

  fn config(&self) -> Configuration;

  fn activity_name(&self) -> String;
}

impl WindowExtAndroid for Window {
  fn content_rect(&self) -> Rect {
    self.window.content_rect()
  }

  fn config(&self) -> Configuration {
    self.window.config()
  }

  fn activity_name(&self) -> String {
    self.window.activity_name().to_string()
  }
}

impl<T> EventLoopWindowTargetExtAndroid for EventLoopWindowTarget<T> {}

/// Additional methods on `WindowBuilder` that are specific to Android.
pub trait WindowBuilderExtAndroid {
  /// The name of the activity class to create.
  fn with_activity_name(self, activity_name: String) -> Self;

  /// The name of the activity class that created this window.
  ///
  /// This is important to define which stack the activity will be created on.
  fn with_created_by_activity_name(self, created_by_activity_name: String) -> Self;
}

impl WindowBuilderExtAndroid for WindowBuilder {
  fn with_activity_name(mut self, activity_name: String) -> Self {
    self.platform_specific.activity_name = Some(activity_name);
    self
  }

  fn with_created_by_activity_name(mut self, created_by_activity_name: String) -> Self {
    self.platform_specific.created_by_activity_name = Some(created_by_activity_name);
    self
  }
}
