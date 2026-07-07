// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::fmt;

use keyboard_types::{Key, Modifiers};
use windows_sys::Win32::UI::{
    Input::KeyboardAndMouse::*,
    WindowsAndMessaging::{ACCEL, FALT, FCONTROL, FSHIFT, FVIRTKEY},
};

use crate::accelerator::{Accelerator, AcceleratorParseError, KeyAccelerator};

impl KeyAccelerator {
    pub fn to_accel(&self, menu_id: u16) -> crate::Result<ACCEL> {
        let mut virt_key = FVIRTKEY;
        let key_mods: Modifiers = self.mods;
        if key_mods.contains(Modifiers::CONTROL) {
            virt_key |= FCONTROL;
        }
        if key_mods.contains(Modifiers::ALT) {
            virt_key |= FALT;
        }
        if key_mods.contains(Modifiers::SHIFT) {
            virt_key |= FSHIFT;
        }

        let vk_code = key_to_vk(&self.key)?;
        let mod_code = vk_code >> 8;
        if mod_code & 0x1 != 0 {
            virt_key |= FSHIFT;
        }
        if mod_code & 0x02 != 0 {
            virt_key |= FCONTROL;
        }
        if mod_code & 0x04 != 0 {
            virt_key |= FALT;
        }
        let raw_key = vk_code & 0x00ff;

        Ok(ACCEL {
            fVirt: virt_key,
            key: raw_key,
            cmd: menu_id,
        })
    }
}

// used to build accelerators table from Key
fn key_to_vk(key: &Key) -> Result<VIRTUAL_KEY, AcceleratorParseError> {
    Ok(match key {
        Key::Character(s) => match s.as_str() {
            "a" | "A" => VK_A,
            "b" | "B" => VK_B,
            "c" | "C" => VK_C,
            "d" | "D" => VK_D,
            "e" | "E" => VK_E,
            "f" | "F" => VK_F,
            "g" | "G" => VK_G,
            "h" | "H" => VK_H,
            "i" | "I" => VK_I,
            "j" | "J" => VK_J,
            "k" | "K" => VK_K,
            "l" | "L" => VK_L,
            "m" | "M" => VK_M,
            "n" | "N" => VK_N,
            "o" | "O" => VK_O,
            "p" | "P" => VK_P,
            "q" | "Q" => VK_Q,
            "r" | "R" => VK_R,
            "s" | "S" => VK_S,
            "t" | "T" => VK_T,
            "u" | "U" => VK_U,
            "v" | "V" => VK_V,
            "w" | "W" => VK_W,
            "x" | "X" => VK_X,
            "y" | "Y" => VK_Y,
            "z" | "Z" => VK_Z,
            "0" => VK_0,
            "1" => VK_1,
            "2" => VK_2,
            "3" => VK_3,
            "4" => VK_4,
            "5" => VK_5,
            "6" => VK_6,
            "7" => VK_7,
            "8" => VK_8,
            "9" => VK_9,
            "=" | "+" => VK_OEM_PLUS,
            "," => VK_OEM_COMMA,
            "-" => VK_OEM_MINUS,
            "." => VK_OEM_PERIOD,
            ";" => VK_OEM_1,
            "/" => VK_OEM_2,
            "`" => VK_OEM_3,
            "[" => VK_OEM_4,
            "\\" => VK_OEM_5,
            "]" => VK_OEM_6,
            "'" => VK_OEM_7,
            " " => VK_SPACE,
            other => {
                // Try VkKeyScanW for characters reachable on the current keyboard layout
                let c = other
                    .chars()
                    .next()
                    .ok_or_else(|| AcceleratorParseError::UnsupportedKey(other.to_string()))?;
                let result = unsafe { VkKeyScanW(c as u16) };
                if result as i16 == -1 {
                    return Err(AcceleratorParseError::UnsupportedKey(other.to_string()));
                }
                result as VIRTUAL_KEY
            }
        },
        Key::Backspace => VK_BACK,
        Key::Tab => VK_TAB,
        Key::Enter => VK_RETURN,
        Key::Pause => VK_PAUSE,
        Key::CapsLock => VK_CAPITAL,
        Key::KanaMode => VK_KANA,
        Key::Escape => VK_ESCAPE,
        Key::NonConvert => VK_NONCONVERT,
        Key::PageUp => VK_PRIOR,
        Key::PageDown => VK_NEXT,
        Key::End => VK_END,
        Key::Home => VK_HOME,
        Key::ArrowLeft => VK_LEFT,
        Key::ArrowUp => VK_UP,
        Key::ArrowRight => VK_RIGHT,
        Key::ArrowDown => VK_DOWN,
        Key::PrintScreen => VK_SNAPSHOT,
        Key::Insert => VK_INSERT,
        Key::Delete => VK_DELETE,
        Key::Help => VK_HELP,
        Key::ContextMenu => VK_APPS,
        Key::F1 => VK_F1,
        Key::F2 => VK_F2,
        Key::F3 => VK_F3,
        Key::F4 => VK_F4,
        Key::F5 => VK_F5,
        Key::F6 => VK_F6,
        Key::F7 => VK_F7,
        Key::F8 => VK_F8,
        Key::F9 => VK_F9,
        Key::F10 => VK_F10,
        Key::F11 => VK_F11,
        Key::F12 => VK_F12,
        Key::F13 => VK_F13,
        Key::F14 => VK_F14,
        Key::F15 => VK_F15,
        Key::F16 => VK_F16,
        Key::F17 => VK_F17,
        Key::F18 => VK_F18,
        Key::F19 => VK_F19,
        Key::F20 => VK_F20,
        Key::F21 => VK_F21,
        Key::F22 => VK_F22,
        Key::F23 => VK_F23,
        Key::F24 => VK_F24,
        Key::NumLock => VK_NUMLOCK,
        Key::ScrollLock => VK_SCROLL,
        Key::BrowserBack => VK_BROWSER_BACK,
        Key::BrowserForward => VK_BROWSER_FORWARD,
        Key::BrowserRefresh => VK_BROWSER_REFRESH,
        Key::BrowserStop => VK_BROWSER_STOP,
        Key::BrowserSearch => VK_BROWSER_SEARCH,
        Key::BrowserFavorites => VK_BROWSER_FAVORITES,
        Key::BrowserHome => VK_BROWSER_HOME,
        Key::AudioVolumeMute => VK_VOLUME_MUTE,
        Key::AudioVolumeDown => VK_VOLUME_DOWN,
        Key::AudioVolumeUp => VK_VOLUME_UP,
        Key::MediaTrackNext => VK_MEDIA_NEXT_TRACK,
        Key::MediaTrackPrevious => VK_MEDIA_PREV_TRACK,
        Key::MediaStop => VK_MEDIA_STOP,
        Key::MediaPlayPause => VK_MEDIA_PLAY_PAUSE,
        Key::LaunchMail => VK_LAUNCH_MAIL,
        Key::Convert => VK_CONVERT,
        key => return Err(AcceleratorParseError::UnsupportedKey(format!("{:?}", key))),
    })
}

impl fmt::Display for Accelerator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", KeyAccelerator::from(*self))
    }
}

impl fmt::Display for KeyAccelerator {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let key_mods: Modifiers = self.mods;
        if key_mods.contains(Modifiers::CONTROL) {
            write!(f, "Ctrl+")?;
        }
        if key_mods.contains(Modifiers::SHIFT) {
            write!(f, "Shift+")?;
        }
        if key_mods.contains(Modifiers::ALT) {
            write!(f, "Alt+")?;
        }
        if key_mods.contains(Modifiers::SUPER) {
            write!(f, "Windows+")?;
        }
        match &self.key {
            Key::Character(s) => match s.as_str() {
                " " => write!(f, "Space"),
                c => write!(f, "{}", c.to_uppercase()),
            },
            Key::Tab => write!(f, "Tab"),
            Key::Escape => write!(f, "Esc"),
            Key::Delete => write!(f, "Del"),
            Key::Insert => write!(f, "Ins"),
            Key::PageUp => write!(f, "PgUp"),
            Key::PageDown => write!(f, "PgDn"),
            Key::ArrowLeft => write!(f, "Left"),
            Key::ArrowRight => write!(f, "Right"),
            Key::ArrowUp => write!(f, "Up"),
            Key::ArrowDown => write!(f, "Down"),
            key => write!(f, "{:?}", key),
        }
    }
}
