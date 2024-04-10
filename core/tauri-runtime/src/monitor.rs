// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::dpi::{PhysicalPosition, PhysicalSize};

/// Monitor descriptor.
#[derive(Debug, Clone)]
pub struct Monitor {
  /// A human-readable name of the monitor.
  /// `None` if the monitor doesn't exist anymore.
  pub name: Option<String>,
  /// The monitor's resolution.
  pub size: PhysicalSize<u32>,
  /// The top-left corner position of the monitor relative to the larger full screen area.
  pub position: PhysicalPosition<i32>,
  /// Returns the scale factor that can be used to map logical pixels to physical pixels, and vice versa.
  pub scale_factor: f64,
}
