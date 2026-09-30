// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use serde::Serialize;

/// Physical fold posture reported by OpenHarmony, independently of window size.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub enum FoldStatus {
  /// The platform cannot describe the current posture.
  Unknown,
  /// The platform reports an expanded device.
  Expanded,
  /// The platform reports a folded device.
  Folded,
  /// The device is between folded and expanded.
  HalfFolded,
}

/// Device-wide fold information. This does not describe a window's hinge geometry.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FoldState {
  /// Whether the platform identifies this device as foldable.
  pub is_foldable: bool,
  /// Current posture, or unknown for ordinary devices and unrecognized native states.
  pub status: FoldStatus,
}

#[cfg(any(target_env = "ohos", test))]
impl FoldState {
  pub(crate) fn from_native(is_foldable: bool, status: u32) -> Self {
    Self {
      is_foldable,
      status: if !is_foldable {
        FoldStatus::Unknown
      } else {
        match status {
          1 => FoldStatus::Expanded,
          2 => FoldStatus::Folded,
          3 => FoldStatus::HalfFolded,
          _ => FoldStatus::Unknown,
        }
      },
    }
  }
}

#[cfg(any(target_env = "ohos", test))]
pub(crate) mod native {
  use super::FoldState;
  use std::sync::{Arc, Mutex};

  type Listener = Arc<dyn Fn(Option<FoldState>) + Send + Sync>;
  static STATE: Mutex<(Option<FoldState>, Option<Listener>)> = Mutex::new((None, None));

  pub(crate) fn snapshot() -> Option<FoldState> {
    STATE.lock().unwrap().0
  }

  pub(crate) fn set_listener(listener: Option<Listener>) {
    STATE.lock().unwrap().1 = listener;
  }

  pub(crate) fn update(state: Option<FoldState>) {
    let listener = {
      let mut current = STATE.lock().unwrap();
      if current.0 == state {
        return;
      }
      current.0 = state;
      current.1.clone()
    };
    // Emitting may re-enter application code. Never invoke it under the state lock.
    if let Some(listener) = listener {
      listener(state);
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn maps_native_postures_without_guessing_unknown_states() {
    for (raw, status) in [
      (0, FoldStatus::Unknown),
      (1, FoldStatus::Expanded),
      (2, FoldStatus::Folded),
      (3, FoldStatus::HalfFolded),
      (99, FoldStatus::Unknown),
    ] {
      assert_eq!(
        FoldState::from_native(true, raw),
        FoldState {
          is_foldable: true,
          status
        }
      );
      assert_eq!(
        FoldState::from_native(false, raw).status,
        FoldStatus::Unknown
      );
    }
  }

  #[test]
  fn caches_initial_state_deduplicates_and_emits_outside_the_lock() {
    use std::sync::{Arc, Mutex};
    let expanded = Some(FoldState::from_native(true, 1));
    native::update(expanded);
    assert_eq!(native::snapshot(), expanded);
    let events = Arc::new(Mutex::new(Vec::new()));
    let seen = events.clone();
    native::set_listener(Some(Arc::new(move |state| {
      assert_eq!(native::snapshot(), state); // would deadlock under the state lock
      seen.lock().unwrap().push(state);
    })));
    native::update(expanded);
    let folded = Some(FoldState::from_native(true, 2));
    native::update(folded);
    native::update(None);
    native::set_listener(None);
    native::update(expanded);
    assert_eq!(*events.lock().unwrap(), vec![folded, None]);
    native::update(None);
  }

  #[test]
  fn serializes_the_public_javascript_contract() {
    assert_eq!(
      serde_json::to_value(FoldState::from_native(true, 3)).unwrap(),
      serde_json::json!({ "isFoldable": true, "status": "halfFolded" })
    );
    assert_eq!(
      serde_json::to_value(None::<FoldState>).unwrap(),
      serde_json::Value::Null
    );
  }
}
