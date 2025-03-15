use serde::{Deserialize, Serialize};

pub use keyboard_types::{Code, Key, KeyState, Location, Modifiers};

/// Keyboard events are issued for all pressed and released keys.
#[derive(Clone, Debug, Default, Eq, Hash, PartialEq, Serialize, Deserialize)]
pub struct KeyboardEvent {
  /// Whether the key is pressed or released.
  pub state: KeyState,
  /// Logical key value.
  pub key: Key,
  /// Physical key position.
  pub code: Code,
  /// Location for keys with multiple instances on common keyboards.
  pub location: Location,
  /// True if the key is currently auto-repeated.
  pub repeat: bool,
}

impl KeyboardEvent {
  pub fn is_down(&self) -> bool {
    self.state == KeyState::Down
  }

  pub fn is_up(&self) -> bool {
    self.state == KeyState::Up
  }
}
