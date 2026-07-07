// Copyright 2014-2021 The winit contributors
// Copyright 2021-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0

use super::KeyEventExtra;
use crate::{
  event::{ElementState, KeyEvent},
  keyboard::{Key, KeyCode, KeyLocation, ModifiersState, NativeKeyCode},
};
use gtk4::{gdk, prelude::DisplayExtManual};
use once_cell::sync::Lazy;
use std::{collections::HashSet, sync::Mutex};

pub type RawKey = gdk::Key;

static KEY_STRINGS: Lazy<Mutex<HashSet<&'static str>>> = Lazy::new(|| Mutex::new(HashSet::new()));

fn insert_or_get_key_str(string: String) -> &'static str {
  let mut string_set = KEY_STRINGS.lock().unwrap();
  if let Some(contained) = string_set.get(string.as_str()) {
    return contained;
  }
  let static_str = Box::leak(string.into_boxed_str());
  string_set.insert(static_str);
  static_str
}

#[allow(clippy::just_underscores_and_digits, non_upper_case_globals)]
pub(crate) fn raw_key_to_key(gdk_key: RawKey) -> Option<Key<'static>> {
  match gdk_key {
    RawKey::Escape => Some(Key::Escape),
    RawKey::BackSpace => Some(Key::Backspace),
    RawKey::Tab | RawKey::ISO_Left_Tab => Some(Key::Tab),
    RawKey::Return => Some(Key::Enter),
    RawKey::Control_L | RawKey::Control_R => Some(Key::Control),
    RawKey::Alt_L | RawKey::Alt_R => Some(Key::Alt),
    RawKey::Shift_L | RawKey::Shift_R => Some(Key::Shift),
    // TODO: investigate mapping. Map Meta_[LR]?
    RawKey::Super_L | RawKey::Super_R => Some(Key::Super),
    RawKey::Caps_Lock => Some(Key::CapsLock),
    RawKey::F1 => Some(Key::F1),
    RawKey::F2 => Some(Key::F2),
    RawKey::F3 => Some(Key::F3),
    RawKey::F4 => Some(Key::F4),
    RawKey::F5 => Some(Key::F5),
    RawKey::F6 => Some(Key::F6),
    RawKey::F7 => Some(Key::F7),
    RawKey::F8 => Some(Key::F8),
    RawKey::F9 => Some(Key::F9),
    RawKey::F10 => Some(Key::F10),
    RawKey::F11 => Some(Key::F11),
    RawKey::F12 => Some(Key::F12),
    RawKey::F13 => Some(Key::F13),
    RawKey::F14 => Some(Key::F14),
    RawKey::F15 => Some(Key::F15),
    RawKey::F16 => Some(Key::F16),
    RawKey::F17 => Some(Key::F17),
    RawKey::F18 => Some(Key::F18),
    RawKey::F19 => Some(Key::F19),
    RawKey::F20 => Some(Key::F20),
    RawKey::F21 => Some(Key::F21),
    RawKey::F22 => Some(Key::F22),
    RawKey::F23 => Some(Key::F23),
    RawKey::F24 => Some(Key::F24),

    RawKey::Print => Some(Key::PrintScreen),
    RawKey::Scroll_Lock => Some(Key::ScrollLock),
    // Pause/Break not audio.
    RawKey::Pause => Some(Key::Pause),

    RawKey::Insert => Some(Key::Insert),
    RawKey::Delete => Some(Key::Delete),
    RawKey::Home => Some(Key::Home),
    RawKey::End => Some(Key::End),
    RawKey::Page_Up => Some(Key::PageUp),
    RawKey::Page_Down => Some(Key::PageDown),
    RawKey::Num_Lock => Some(Key::NumLock),

    RawKey::Up => Some(Key::ArrowUp),
    RawKey::Down => Some(Key::ArrowDown),
    RawKey::Left => Some(Key::ArrowLeft),
    RawKey::Right => Some(Key::ArrowRight),
    RawKey::Clear => Some(Key::Clear),

    RawKey::Menu => Some(Key::ContextMenu),
    RawKey::WakeUp => Some(Key::WakeUp),
    RawKey::Launch0 => Some(Key::LaunchApplication1),
    RawKey::Launch1 => Some(Key::LaunchApplication2),
    RawKey::ISO_Level3_Shift => Some(Key::AltGraph),

    RawKey::KP_Begin => Some(Key::Clear),
    RawKey::KP_Delete => Some(Key::Delete),
    RawKey::KP_Down => Some(Key::ArrowDown),
    RawKey::KP_End => Some(Key::End),
    RawKey::KP_Enter => Some(Key::Enter),
    RawKey::KP_F1 => Some(Key::F1),
    RawKey::KP_F2 => Some(Key::F2),
    RawKey::KP_F3 => Some(Key::F3),
    RawKey::KP_F4 => Some(Key::F4),
    RawKey::KP_Home => Some(Key::Home),
    RawKey::KP_Insert => Some(Key::Insert),
    RawKey::KP_Left => Some(Key::ArrowLeft),
    RawKey::KP_Page_Down => Some(Key::PageDown),
    RawKey::KP_Page_Up => Some(Key::PageUp),
    RawKey::KP_Right => Some(Key::ArrowRight),
    // KP_Separator? What does it map to?
    RawKey::KP_Tab => Some(Key::Tab),
    RawKey::KP_Up => Some(Key::ArrowUp),

    // JIS
    RawKey::Zenkaku_Hankaku => Some(Key::ZenkakuHankaku),
    RawKey::Hiragana_Katakana => Some(Key::HiraganaKatakana),
    RawKey::Henkan => Some(Key::Convert),
    RawKey::Muhenkan => Some(Key::NonConvert),
    // TODO: more mappings (media etc)
    _ => None,
  }
}

#[allow(clippy::just_underscores_and_digits, non_upper_case_globals)]
pub(crate) fn raw_key_to_location(raw: RawKey) -> KeyLocation {
  match raw {
    RawKey::Control_L | RawKey::Shift_L | RawKey::Alt_L | RawKey::Super_L | RawKey::Meta_L => {
      KeyLocation::Left
    }
    RawKey::Control_R | RawKey::Shift_R | RawKey::Alt_R | RawKey::Super_R | RawKey::Meta_R => {
      KeyLocation::Right
    }
    RawKey::KP_0
    | RawKey::KP_1
    | RawKey::KP_2
    | RawKey::KP_3
    | RawKey::KP_4
    | RawKey::KP_5
    | RawKey::KP_6
    | RawKey::KP_7
    | RawKey::KP_8
    | RawKey::KP_9
    | RawKey::KP_Add
    | RawKey::KP_Begin
    | RawKey::KP_Decimal
    | RawKey::KP_Delete
    | RawKey::KP_Divide
    | RawKey::KP_Down
    | RawKey::KP_End
    | RawKey::KP_Enter
    | RawKey::KP_Equal
    | RawKey::KP_F1
    | RawKey::KP_F2
    | RawKey::KP_F3
    | RawKey::KP_F4
    | RawKey::KP_Home
    | RawKey::KP_Insert
    | RawKey::KP_Left
    | RawKey::KP_Multiply
    | RawKey::KP_Page_Down
    | RawKey::KP_Page_Up
    | RawKey::KP_Right
    | RawKey::KP_Separator
    | RawKey::KP_Space
    | RawKey::KP_Subtract
    | RawKey::KP_Tab
    | RawKey::KP_Up => KeyLocation::Numpad,
    _ => KeyLocation::Standard,
  }
}

const MODIFIER_MAP: &[(Key<'static>, ModifiersState)] = &[
  (Key::Shift, ModifiersState::SHIFT),
  (Key::Alt, ModifiersState::ALT),
  (Key::Control, ModifiersState::CONTROL),
  (Key::Super, ModifiersState::SUPER),
];

// we use the EventKey to extract the modifier mainly because
// we need to have the modifier before the second key is entered to follow
// other os' logic -- this way we can emit the new `ModifiersState` before
// we receive the next key, if needed the developer can update his local state.
pub(crate) fn get_modifiers(key: RawKey, scancode: u16) -> ModifiersState {
  // a keycode (scancode in Windows) is a code that refers to a physical keyboard key.
  // unicode value
  let unicode = key.to_unicode();
  // translate to tao::keyboard::Key
  let key_from_code = raw_key_to_key(key).unwrap_or_else(|| {
    if let Some(key) = unicode {
      if key >= ' ' && key != '\x7f' {
        Key::Character(insert_or_get_key_str(key.to_string()))
      } else {
        Key::Unidentified(NativeKeyCode::Gtk(scancode))
      }
    } else {
      Key::Unidentified(NativeKeyCode::Gtk(scancode))
    }
  });
  // start with empty state
  let mut result = ModifiersState::empty();
  // loop trough our modifier map
  for (gdk_mod, modifier) in MODIFIER_MAP {
    if key_from_code == *gdk_mod {
      result |= *modifier;
    }
  }
  result
}

pub(crate) fn make_key_event(
  key: &RawKey,
  scancode: u16,
  is_repeat: bool,
  key_override: Option<KeyCode>,
  state: ElementState,
) -> Option<KeyEvent> {
  // a keycode (scancode in Windows) is a code that refers to a physical keyboard key.
  let keyval_with_modifiers = hardware_keycode_to_keyval(scancode).unwrap_or(*key);
  // get unicode value, with and without modifiers
  let text_without_modifiers = keyval_with_modifiers.to_unicode();
  let text_with_modifiers = key.to_unicode();
  // get physical key from the scancode (keycode)
  let physical_key = key_override.unwrap_or_else(|| KeyCode::from_scancode(scancode as u32));

  // extract key without modifier
  let key_without_modifiers = raw_key_to_key(keyval_with_modifiers).unwrap_or_else(|| {
    if let Some(key) = text_without_modifiers {
      if key >= ' ' && key != '\x7f' {
        Key::Character(insert_or_get_key_str(key.to_string()))
      } else {
        Key::Unidentified(NativeKeyCode::Gtk(scancode))
      }
    } else {
      Key::Unidentified(NativeKeyCode::Gtk(scancode))
    }
  });

  // extract the logical key
  let logical_key = raw_key_to_key(*key).unwrap_or_else(|| {
    if let Some(key) = text_with_modifiers {
      if key >= ' ' && key != '\x7f' {
        Key::Character(insert_or_get_key_str(key.to_string()))
      } else {
        Key::Unidentified(NativeKeyCode::Gtk(scancode))
      }
    } else {
      Key::Unidentified(NativeKeyCode::Gtk(scancode))
    }
  });

  // make sure we have a valid key
  if !matches!(key_without_modifiers, Key::Unidentified(_)) {
    let location = raw_key_to_location(keyval_with_modifiers);
    let text_with_all_modifiers =
      text_without_modifiers.map(|text| insert_or_get_key_str(text.to_string()));
    return Some(KeyEvent {
      location,
      logical_key,
      physical_key,
      repeat: is_repeat,
      state,
      text: text_with_all_modifiers,
      platform_specific: KeyEventExtra {
        text_with_all_modifiers,
        key_without_modifiers,
      },
    });
  } else {
    #[cfg(debug_assertions)]
    eprintln!("Couldn't get key from code: {physical_key:?}");
  }
  None
}

/// Map a hardware keycode to a keyval by performing a lookup in the keymap and finding the
/// keyval with the lowest group and level
fn hardware_keycode_to_keyval(keycode: u16) -> Option<RawKey> {
  let display = gdk::Display::default()?;
  let keymap = display.map_keycode(keycode.into())?;

  keymap.iter().find_map(|(keymap_key, key)| {
    if keymap_key.group() == 0 && keymap_key.level() == 0 {
      Some(*key)
    } else {
      None
    }
  })
}
