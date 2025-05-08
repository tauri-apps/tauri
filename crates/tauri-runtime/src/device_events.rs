use dpi::PhysicalPosition;
pub use keyboard_types::{Code, KeyState};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(tag = "type")]
pub enum DeviceEventFilter {
  /// Always filter out device events.
  Always,
  /// Filter out device events while the window is not focused.
  Unfocused,
  /// Report all device events regardless of window focus.
  Never,
}

impl Default for DeviceEventFilter {
  fn default() -> Self {
    Self::Unfocused
  }
}

pub trait DeviceId: Copy + Clone + PartialEq + Eq + PartialOrd + Ord + std::hash::Hash {
  /// # Safety
  /// Returns a dummy `DeviceId`, useful for unit testing. The only guarantee made about the return
  /// value of this function is that it will always be equal to itself and to future values returned
  /// by this function.  No other guarantees are made. This may be equal to a real `DeviceId`.
  ///
  /// **Passing this id to any real device will result in undefined behavior.**
  unsafe fn dummy() -> Self;
}

#[derive(Clone, Debug, PartialEq)]
pub enum DeviceEvent {
  Added,
  Removed,

  /// Change in physical position of a pointing device.
  ///
  /// This represents raw, unfiltered physical motion. Not to be confused with `WindowEvent::CursorMoved`.
  MouseMotion {
    /// (x, y) change in position in unspecified units.
    ///
    /// Different devices may use different units.
    delta: (f64, f64),
  },

  /// Physical scroll event
  MouseWheel {
    delta: MouseScrollDelta,
  },

  /// Motion on some analog axis.  This event will be reported for all arbitrary input devices
  /// that tao supports on this platform, including mouse devices.  If the device is a mouse
  /// device then this will be reported alongside the MouseMotion event.
  Motion {
    axis: AxisId,
    value: f64,
  },

  Button {
    button: ButtonId,
    state: KeyState,
  },

  Key {
    pysical_key: Code,
    state: KeyState,
  },

  Text {
    codepoint: char,
  },
}

/// Identifier for a specific analog axis on some device.
pub type AxisId = u32;

/// Identifier for a specific button on some device.
///
/// For a mouse, this is the button number (0 for left, 1 for right, etc.).
pub type ButtonId = u32;

/// Describes a difference in the mouse scroll wheel state.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum MouseScrollDelta {
  /// Amount in lines or rows to scroll in the horizontal
  /// and vertical directions.
  ///
  /// Positive values indicate movement forward
  /// (away from the user) or rightwards.
  LineDelta(f32, f32),
  /// Amount in pixels to scroll in the horizontal and
  /// vertical direction.
  ///
  /// Scroll events are expressed as a PixelDelta if
  /// supported by the device (eg. a touchpad) and
  /// platform.
  PixelDelta(PhysicalPosition<f64>),
}
