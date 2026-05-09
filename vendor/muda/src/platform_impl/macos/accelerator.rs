// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use keyboard_types::{Key, Modifiers};
use objc2_app_kit::NSEventModifierFlags;

use crate::accelerator::{AcceleratorParseError, KeyAccelerator};

impl KeyAccelerator {
    /// Return the string value of this hotkey, without modifiers.
    pub(crate) fn key_equivalent(&self) -> Result<String, AcceleratorParseError> {
        Ok(match &self.key {
            Key::Character(s) => s.clone(),
            Key::Tab => "⇥".into(),
            Key::Escape => "\u{001b}".into(),
            Key::Enter => "\u{0003}".into(),
            Key::Backspace => "\u{0008}".into(),
            Key::Delete => "\u{007f}".into(),
            Key::Insert => "\u{F727}".into(),
            Key::Home => "\u{F729}".into(),
            Key::End => "\u{F72B}".into(),
            Key::PageUp => "\u{F72C}".into(),
            Key::PageDown => "\u{F72D}".into(),
            Key::PrintScreen => "\u{F72E}".into(),
            Key::ScrollLock => "\u{F72F}".into(),
            Key::ArrowUp => "\u{F700}".into(),
            Key::ArrowDown => "\u{F701}".into(),
            Key::ArrowLeft => "\u{F702}".into(),
            Key::ArrowRight => "\u{F703}".into(),
            Key::F1 => "\u{F704}".into(),
            Key::F2 => "\u{F705}".into(),
            Key::F3 => "\u{F706}".into(),
            Key::F4 => "\u{F707}".into(),
            Key::F5 => "\u{F708}".into(),
            Key::F6 => "\u{F709}".into(),
            Key::F7 => "\u{F70A}".into(),
            Key::F8 => "\u{F70B}".into(),
            Key::F9 => "\u{F70C}".into(),
            Key::F10 => "\u{F70D}".into(),
            Key::F11 => "\u{F70E}".into(),
            Key::F12 => "\u{F70F}".into(),
            Key::F13 => "\u{F710}".into(),
            Key::F14 => "\u{F711}".into(),
            Key::F15 => "\u{F712}".into(),
            Key::F16 => "\u{F713}".into(),
            Key::F17 => "\u{F714}".into(),
            Key::F18 => "\u{F715}".into(),
            Key::F19 => "\u{F716}".into(),
            Key::F20 => "\u{F717}".into(),
            Key::F21 => "\u{F718}".into(),
            Key::F22 => "\u{F719}".into(),
            Key::F23 => "\u{F71A}".into(),
            Key::F24 => "\u{F71B}".into(),
            key => return Err(AcceleratorParseError::UnsupportedKey(format!("{:?}", key))),
        })
    }

    /// Return the modifiers of this hotkey, as an NSEventModifierFlags bitflag.
    pub(crate) fn modifier_mask(&self) -> NSEventModifierFlags {
        let mut flags = NSEventModifierFlags::empty();
        if self.mods.contains(Modifiers::SHIFT) {
            flags.insert(NSEventModifierFlags::Shift);
        }
        if self.mods.contains(Modifiers::SUPER) {
            flags.insert(NSEventModifierFlags::Command);
        }
        if self.mods.contains(Modifiers::ALT) {
            flags.insert(NSEventModifierFlags::Option);
        }
        if self.mods.contains(Modifiers::CONTROL) {
            flags.insert(NSEventModifierFlags::Control);
        }
        flags
    }
}
