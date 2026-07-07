// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Accelerators describe keyboard shortcuts for menu items.
//!
//! [`KeyAccelerator`s](crate::accelerator::KeyAccelerator) are used to define a keyboard
//! shortcut based on logical keys, which allows expressing shortcuts like `Ctrl++` or `Ctrl+€`
//! that physical key codes cannot represent.
//! For this reason, prefer to use [`KeyAccelerator`s](crate::accelerator::KeyAccelerator) over the older [`Accelerator`s](crate::accelerator::Accelerator).
//!
//! # Examples
//! They can be created directly
//! ```no_run
//! # use muda::accelerator::*;
//! let key_accelerator = KeyAccelerator::new(Some(Modifiers::SHIFT), Key::Character("q".to_owned()));
//! let key_accelerator_without_mods = KeyAccelerator::new(None, Key::Character("q".to_owned()));
//!
//! let accelerator = Accelerator::new(Some(Modifiers::SHIFT), Code::KeyQ);
//! let accelerator_without_mods = Accelerator::new(None, Code::KeyQ);
//! ```
//! or from `&str`, note that all modifiers
//! have to be listed before the non-modifier key, `shift+alt+KeyQ` is legal,
//! whereas `shift+q+alt` is not.
//! ```no_run
//! # use muda::accelerator::*;
//! let key_accelerator: KeyAccelerator = "shift+alt+KeyQ".parse().unwrap();
//! // Or alternatively:
//! let key_accelerator: KeyAccelerator = "shift+alt+q".parse().unwrap();
//!
//! // Or to parse Accelerator
//! let accelerator: Accelerator = "shift+alt+KeyQ".parse().unwrap();
//! let accelerator: Accelerator = "shift+alt+q".parse().unwrap();
//! # // This assert exists to ensure a test breaks once the
//! # // statement above about ordering is no longer valid.
//! # assert!("shift+KeyQ+alt".parse::<KeyAccelerator>().is_err());
//! # assert!("shift+KeyQ+alt".parse::<Accelerator>().is_err());
//! ```

pub use keyboard_types::{Code, Key, Modifiers};
use std::{borrow::Borrow, hash::Hash, str::FromStr};

#[cfg(target_os = "macos")]
pub const CMD_OR_CTRL: Modifiers = Modifiers::SUPER;
#[cfg(not(target_os = "macos"))]
pub const CMD_OR_CTRL: Modifiers = Modifiers::CONTROL;

#[derive(thiserror::Error, Debug, PartialEq, Eq)]
pub enum AcceleratorParseError {
    #[error("Couldn't recognize \"{0}\" as a valid key for accelerator, if you feel like it should be, please report this to https://github.com/tauri-apps/muda")]
    UnsupportedKey(String),
    #[error("Found empty token while parsing accelerator: {0}")]
    EmptyToken(String),
    #[error("Invalid accelerator format: \"{0}\", an accelerator should have the modifiers first and only one main key, for example: \"Shift + Alt + K\"")]
    InvalidFormat(String),
}

/// A keyboard shortcut that consists of an optional combination
/// of modifier keys (provided by [`Modifiers`] and
/// one key ([`Code`]).
///
/// ## Warning
///
/// This struct cannot represent all shortcuts found on non-U.S. keyboard layouts and might
/// be deprecated in the future.
/// Please use [`KeyAccelerator`] instead.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Accelerator {
    pub(crate) mods: Modifiers,
    pub(crate) key: Code,
    id: u32,
}

impl Accelerator {
    /// Creates a new accelerator to define keyboard shortcuts throughout your application.
    /// Only [`Modifiers::ALT`], [`Modifiers::SHIFT`], [`Modifiers::CONTROL`], and [`Modifiers::SUPER`]
    pub fn new(mods: Option<Modifiers>, key: Code) -> Self {
        let mut mods = mods.unwrap_or_else(Modifiers::empty);
        if mods.contains(Modifiers::META) {
            mods.remove(Modifiers::META);
            mods.insert(Modifiers::SUPER);
        }

        let id = Self::generate_hash(mods, key);

        Self { mods, key, id }
    }

    fn generate_hash(mods: Modifiers, key: Code) -> u32 {
        KeyAccelerator::generate_hash(mods, &code_to_key(key))
    }

    /// Returns the id associated with this accelerator
    /// which is a hash of the string representation of modifiers and key within this accelerator.
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Returns the modifier for this accelerator
    pub fn modifiers(&self) -> Modifiers {
        self.mods
    }

    /// Returns the code for this accelerator
    pub fn key(&self) -> Code {
        self.key
    }

    /// Returns `true` if this [`Code`] and [`Modifiers`] matches this `Accelerator`.
    pub fn matches(&self, modifiers: impl Borrow<Modifiers>, key: impl Borrow<Code>) -> bool {
        // Should be a const but const bit_or doesn't work here.
        let base_mods = Modifiers::SHIFT | Modifiers::CONTROL | Modifiers::ALT | Modifiers::SUPER;
        let modifiers = modifiers.borrow();
        let key = key.borrow();
        self.mods == *modifiers & base_mods && self.key == *key
    }
}

impl FromStr for Accelerator {
    type Err = AcceleratorParseError;
    fn from_str(accelerator_string: &str) -> Result<Self, Self::Err> {
        parse_accelerator(accelerator_string)
    }
}

impl TryFrom<&str> for Accelerator {
    type Error = AcceleratorParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        parse_accelerator(value)
    }
}

impl TryFrom<String> for Accelerator {
    type Error = AcceleratorParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        parse_accelerator(&value)
    }
}

fn parse_accelerator(accelerator: &str) -> Result<Accelerator, AcceleratorParseError> {
    let (modifiers_str, code_str) = split_key_and_modifiers(accelerator)?;
    let modifiers = parse_modifiers(accelerator, modifiers_str)?;
    let code = parse_code(code_str)?;

    Ok(Accelerator::new(Some(modifiers), code))
}

fn parse_code(code: &str) -> Result<Code, AcceleratorParseError> {
    use Code::*;
    match code.to_uppercase().as_str() {
        "BACKQUOTE" | "`" => Ok(Backquote),
        "BACKSLASH" | "\\" => Ok(Backslash),
        "BRACKETLEFT" | "[" => Ok(BracketLeft),
        "BRACKETRIGHT" | "]" => Ok(BracketRight),
        "COMMA" | "," => Ok(Comma),
        "DIGIT0" | "0" => Ok(Digit0),
        "DIGIT1" | "1" => Ok(Digit1),
        "DIGIT2" | "2" => Ok(Digit2),
        "DIGIT3" | "3" => Ok(Digit3),
        "DIGIT4" | "4" => Ok(Digit4),
        "DIGIT5" | "5" => Ok(Digit5),
        "DIGIT6" | "6" => Ok(Digit6),
        "DIGIT7" | "7" => Ok(Digit7),
        "DIGIT8" | "8" => Ok(Digit8),
        "DIGIT9" | "9" => Ok(Digit9),
        "EQUAL" | "=" => Ok(Equal),
        "KEYA" | "A" => Ok(KeyA),
        "KEYB" | "B" => Ok(KeyB),
        "KEYC" | "C" => Ok(KeyC),
        "KEYD" | "D" => Ok(KeyD),
        "KEYE" | "E" => Ok(KeyE),
        "KEYF" | "F" => Ok(KeyF),
        "KEYG" | "G" => Ok(KeyG),
        "KEYH" | "H" => Ok(KeyH),
        "KEYI" | "I" => Ok(KeyI),
        "KEYJ" | "J" => Ok(KeyJ),
        "KEYK" | "K" => Ok(KeyK),
        "KEYL" | "L" => Ok(KeyL),
        "KEYM" | "M" => Ok(KeyM),
        "KEYN" | "N" => Ok(KeyN),
        "KEYO" | "O" => Ok(KeyO),
        "KEYP" | "P" => Ok(KeyP),
        "KEYQ" | "Q" => Ok(KeyQ),
        "KEYR" | "R" => Ok(KeyR),
        "KEYS" | "S" => Ok(KeyS),
        "KEYT" | "T" => Ok(KeyT),
        "KEYU" | "U" => Ok(KeyU),
        "KEYV" | "V" => Ok(KeyV),
        "KEYW" | "W" => Ok(KeyW),
        "KEYX" | "X" => Ok(KeyX),
        "KEYY" | "Y" => Ok(KeyY),
        "KEYZ" | "Z" => Ok(KeyZ),
        "MINUS" | "-" => Ok(Minus),
        "PERIOD" | "." => Ok(Period),
        "QUOTE" | "'" => Ok(Quote),
        "SEMICOLON" | ";" => Ok(Semicolon),
        "SLASH" | "/" => Ok(Slash),
        "BACKSPACE" => Ok(Backspace),
        "CAPSLOCK" => Ok(CapsLock),
        "ENTER" => Ok(Enter),
        "SPACE" => Ok(Space),
        "TAB" => Ok(Tab),
        "DELETE" => Ok(Delete),
        "END" => Ok(End),
        "HOME" => Ok(Home),
        "INSERT" => Ok(Insert),
        "PAGEDOWN" => Ok(PageDown),
        "PAGEUP" => Ok(PageUp),
        "PRINTSCREEN" => Ok(PrintScreen),
        "SCROLLLOCK" => Ok(ScrollLock),
        "ARROWDOWN" | "DOWN" => Ok(ArrowDown),
        "ARROWLEFT" | "LEFT" => Ok(ArrowLeft),
        "ARROWRIGHT" | "RIGHT" => Ok(ArrowRight),
        "ARROWUP" | "UP" => Ok(ArrowUp),
        "NUMLOCK" => Ok(NumLock),
        "NUMPAD0" | "NUM0" => Ok(Numpad0),
        "NUMPAD1" | "NUM1" => Ok(Numpad1),
        "NUMPAD2" | "NUM2" => Ok(Numpad2),
        "NUMPAD3" | "NUM3" => Ok(Numpad3),
        "NUMPAD4" | "NUM4" => Ok(Numpad4),
        "NUMPAD5" | "NUM5" => Ok(Numpad5),
        "NUMPAD6" | "NUM6" => Ok(Numpad6),
        "NUMPAD7" | "NUM7" => Ok(Numpad7),
        "NUMPAD8" | "NUM8" => Ok(Numpad8),
        "NUMPAD9" | "NUM9" => Ok(Numpad9),
        "NUMPADADD" | "NUMADD" | "NUMPADPLUS" | "NUMPLUS" => Ok(NumpadAdd),
        "NUMPADDECIMAL" | "NUMDECIMAL" => Ok(NumpadDecimal),
        "NUMPADDIVIDE" | "NUMDIVIDE" => Ok(NumpadDivide),
        "NUMPADENTER" | "NUMENTER" => Ok(NumpadEnter),
        "NUMPADEQUAL" | "NUMEQUAL" => Ok(NumpadEqual),
        "NUMPADMULTIPLY" | "NUMMULTIPLY" => Ok(NumpadMultiply),
        "NUMPADSUBTRACT" | "NUMSUBTRACT" => Ok(NumpadSubtract),
        "ESCAPE" | "ESC" => Ok(Escape),
        "F1" => Ok(F1),
        "F2" => Ok(F2),
        "F3" => Ok(F3),
        "F4" => Ok(F4),
        "F5" => Ok(F5),
        "F6" => Ok(F6),
        "F7" => Ok(F7),
        "F8" => Ok(F8),
        "F9" => Ok(F9),
        "F10" => Ok(F10),
        "F11" => Ok(F11),
        "F12" => Ok(F12),
        "AUDIOVOLUMEDOWN" | "VOLUMEDOWN" => Ok(AudioVolumeDown),
        "AUDIOVOLUMEUP" | "VOLUMEUP" => Ok(AudioVolumeUp),
        "AUDIOVOLUMEMUTE" | "VOLUMEMUTE" => Ok(AudioVolumeMute),
        "F13" => Ok(F13),
        "F14" => Ok(F14),
        "F15" => Ok(F15),
        "F16" => Ok(F16),
        "F17" => Ok(F17),
        "F18" => Ok(F18),
        "F19" => Ok(F19),
        "F20" => Ok(F20),
        "F21" => Ok(F21),
        "F22" => Ok(F22),
        "F23" => Ok(F23),
        "F24" => Ok(F24),

        _ => Err(AcceleratorParseError::UnsupportedKey(code.to_string())),
    }
}

fn code_to_key(code: Code) -> Key {
    match code {
        Code::KeyA => Key::Character("a".into()),
        Code::KeyB => Key::Character("b".into()),
        Code::KeyC => Key::Character("c".into()),
        Code::KeyD => Key::Character("d".into()),
        Code::KeyE => Key::Character("e".into()),
        Code::KeyF => Key::Character("f".into()),
        Code::KeyG => Key::Character("g".into()),
        Code::KeyH => Key::Character("h".into()),
        Code::KeyI => Key::Character("i".into()),
        Code::KeyJ => Key::Character("j".into()),
        Code::KeyK => Key::Character("k".into()),
        Code::KeyL => Key::Character("l".into()),
        Code::KeyM => Key::Character("m".into()),
        Code::KeyN => Key::Character("n".into()),
        Code::KeyO => Key::Character("o".into()),
        Code::KeyP => Key::Character("p".into()),
        Code::KeyQ => Key::Character("q".into()),
        Code::KeyR => Key::Character("r".into()),
        Code::KeyS => Key::Character("s".into()),
        Code::KeyT => Key::Character("t".into()),
        Code::KeyU => Key::Character("u".into()),
        Code::KeyV => Key::Character("v".into()),
        Code::KeyW => Key::Character("w".into()),
        Code::KeyX => Key::Character("x".into()),
        Code::KeyY => Key::Character("y".into()),
        Code::KeyZ => Key::Character("z".into()),
        Code::Digit0 => Key::Character("0".into()),
        Code::Digit1 => Key::Character("1".into()),
        Code::Digit2 => Key::Character("2".into()),
        Code::Digit3 => Key::Character("3".into()),
        Code::Digit4 => Key::Character("4".into()),
        Code::Digit5 => Key::Character("5".into()),
        Code::Digit6 => Key::Character("6".into()),
        Code::Digit7 => Key::Character("7".into()),
        Code::Digit8 => Key::Character("8".into()),
        Code::Digit9 => Key::Character("9".into()),
        Code::Comma => Key::Character(",".into()),
        Code::Minus => Key::Character("-".into()),
        Code::Period => Key::Character(".".into()),
        Code::Space => Key::Character(" ".into()),
        Code::Equal => Key::Character("=".into()),
        Code::Semicolon => Key::Character(";".into()),
        Code::Slash => Key::Character("/".into()),
        Code::Backslash => Key::Character("\\".into()),
        Code::Quote => Key::Character("'".into()),
        Code::Backquote => Key::Character("`".into()),
        Code::BracketLeft => Key::Character("[".into()),
        Code::BracketRight => Key::Character("]".into()),
        Code::Numpad0 => Key::Character("0".into()),
        Code::Numpad1 => Key::Character("1".into()),
        Code::Numpad2 => Key::Character("2".into()),
        Code::Numpad3 => Key::Character("3".into()),
        Code::Numpad4 => Key::Character("4".into()),
        Code::Numpad5 => Key::Character("5".into()),
        Code::Numpad6 => Key::Character("6".into()),
        Code::Numpad7 => Key::Character("7".into()),
        Code::Numpad8 => Key::Character("8".into()),
        Code::Numpad9 => Key::Character("9".into()),
        Code::NumpadAdd => Key::Character("+".into()),
        Code::NumpadDecimal => Key::Character(".".into()),
        Code::NumpadDivide => Key::Character("/".into()),
        Code::NumpadMultiply => Key::Character("*".into()),
        Code::NumpadSubtract => Key::Character("-".into()),
        Code::NumpadEqual => Key::Character("=".into()),
        Code::NumpadEnter => Key::Enter,
        Code::Backspace => Key::Backspace,
        Code::Tab => Key::Tab,
        Code::Enter => Key::Enter,
        Code::Escape => Key::Escape,
        Code::Delete => Key::Delete,
        Code::CapsLock => Key::CapsLock,
        Code::Home => Key::Home,
        Code::End => Key::End,
        Code::PageUp => Key::PageUp,
        Code::PageDown => Key::PageDown,
        Code::Insert => Key::Insert,
        Code::PrintScreen => Key::PrintScreen,
        Code::ScrollLock => Key::ScrollLock,
        Code::NumLock => Key::NumLock,
        Code::Pause => Key::Pause,
        Code::ContextMenu => Key::ContextMenu,
        Code::ArrowUp => Key::ArrowUp,
        Code::ArrowDown => Key::ArrowDown,
        Code::ArrowLeft => Key::ArrowLeft,
        Code::ArrowRight => Key::ArrowRight,
        Code::F1 => Key::F1,
        Code::F2 => Key::F2,
        Code::F3 => Key::F3,
        Code::F4 => Key::F4,
        Code::F5 => Key::F5,
        Code::F6 => Key::F6,
        Code::F7 => Key::F7,
        Code::F8 => Key::F8,
        Code::F9 => Key::F9,
        Code::F10 => Key::F10,
        Code::F11 => Key::F11,
        Code::F12 => Key::F12,
        Code::F13 => Key::F13,
        Code::F14 => Key::F14,
        Code::F15 => Key::F15,
        Code::F16 => Key::F16,
        Code::F17 => Key::F17,
        Code::F18 => Key::F18,
        Code::F19 => Key::F19,
        Code::F20 => Key::F20,
        Code::F21 => Key::F21,
        Code::F22 => Key::F22,
        Code::F23 => Key::F23,
        Code::F24 => Key::F24,
        Code::AudioVolumeDown => Key::AudioVolumeDown,
        Code::AudioVolumeUp => Key::AudioVolumeUp,
        Code::AudioVolumeMute => Key::AudioVolumeMute,
        Code::MediaTrackNext => Key::MediaTrackNext,
        Code::MediaTrackPrevious => Key::MediaTrackPrevious,
        Code::MediaStop => Key::MediaStop,
        Code::MediaPlayPause => Key::MediaPlayPause,
        Code::LaunchMail => Key::LaunchMail,
        Code::BrowserBack => Key::BrowserBack,
        Code::BrowserForward => Key::BrowserForward,
        Code::BrowserHome => Key::BrowserHome,
        Code::BrowserSearch => Key::BrowserSearch,
        Code::BrowserStop => Key::BrowserStop,
        Code::BrowserFavorites => Key::BrowserFavorites,
        Code::BrowserRefresh => Key::BrowserRefresh,
        Code::Convert => Key::Convert,
        Code::KanaMode => Key::KanaMode,
        Code::NonConvert => Key::NonConvert,
        Code::Help => Key::Help,
        // Code is unfortunately non-exhaustive,
        // which means we need to manually notice and update this list if new items are added to Code.
        _ => Key::Unidentified,
    }
}

/// A keyboard shortcut based on logical [`Key`] values.
///
/// Unlike [`Accelerator`] which uses physical [`Code`] keys,
/// `KeyAccelerator` uses logical [`Key`] values which can represent
/// any character including Unicode characters like `+`, `€`, `{`, etc.
///
/// # Examples
///
/// They can be created directly
/// ```no_run
/// # use muda::accelerator::{KeyAccelerator, Modifiers, Key};
/// let accel = KeyAccelerator::new(Some(Modifiers::CONTROL), Key::Character("+".into()));
/// ```
/// or converted from an existing [`Accelerator`]
/// ```no_run
/// # use muda::accelerator::{Accelerator, KeyAccelerator, Code, Modifiers};
/// let old_accel = Accelerator::new(Some(Modifiers::CONTROL), Code::KeyC);
/// let new_accel: KeyAccelerator = old_accel.into();
/// ```
/// or parsed from a string, which supports literal character keys
/// ```no_run
/// # use muda::accelerator::KeyAccelerator;
/// let accel: KeyAccelerator = "Ctrl++".parse().unwrap();
/// let accel2: KeyAccelerator = "Ctrl+€".parse().unwrap();
/// ```
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct KeyAccelerator {
    pub(crate) mods: Modifiers,
    pub(crate) key: Key,
    id: u32,
}

impl KeyAccelerator {
    /// Creates a new key accelerator to define keyboard shortcuts throughout your application.
    /// Only [`Modifiers::ALT`], [`Modifiers::SHIFT`], [`Modifiers::CONTROL`], and [`Modifiers::SUPER`]
    pub fn new(mods: Option<Modifiers>, key: Key) -> Self {
        let mut mods = mods.unwrap_or_else(Modifiers::empty);
        if mods.contains(Modifiers::META) {
            mods.remove(Modifiers::META);
            mods.insert(Modifiers::SUPER);
        }

        let id = Self::generate_hash(mods, &key);

        Self { mods, key, id }
    }

    fn generate_hash(mods: Modifiers, key: &Key) -> u32 {
        let mut accelerator_str = String::new();
        if mods.contains(Modifiers::SHIFT) {
            accelerator_str.push_str("shift+")
        }
        if mods.contains(Modifiers::CONTROL) {
            accelerator_str.push_str("control+")
        }
        if mods.contains(Modifiers::ALT) {
            accelerator_str.push_str("alt+")
        }
        if mods.contains(Modifiers::SUPER) {
            accelerator_str.push_str("super+")
        }
        accelerator_str.push_str(&format!("{:?}", key));

        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        accelerator_str.hash(&mut hasher);
        std::hash::Hasher::finish(&hasher) as u32
    }

    /// Returns the id associated with this accelerator
    /// which is a hash of the string representation of modifiers and key within this accelerator.
    pub fn id(&self) -> u32 {
        self.id
    }

    /// Returns the modifiers for this accelerator.
    pub fn modifiers(&self) -> Modifiers {
        self.mods
    }

    /// Returns the key for this accelerator.
    pub fn key(&self) -> &Key {
        &self.key
    }

    /// Returns `true` if this [`Key`] and [`Modifiers`] matches this `KeyAccelerator`.
    pub fn matches(&self, modifiers: impl Borrow<Modifiers>, key: impl Borrow<Key>) -> bool {
        let base_mods = Modifiers::SHIFT | Modifiers::CONTROL | Modifiers::ALT | Modifiers::SUPER;
        let modifiers = modifiers.borrow();
        let key = key.borrow();
        self.mods == *modifiers & base_mods && self.key == *key
    }
}

impl From<Accelerator> for KeyAccelerator {
    fn from(accel: Accelerator) -> Self {
        KeyAccelerator::new(Some(accel.mods), code_to_key(accel.key))
    }
}

impl FromStr for KeyAccelerator {
    type Err = AcceleratorParseError;
    fn from_str(accelerator_string: &str) -> Result<Self, Self::Err> {
        parse_key_accelerator(accelerator_string)
    }
}

impl TryFrom<&str> for KeyAccelerator {
    type Error = AcceleratorParseError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        parse_key_accelerator(value)
    }
}

impl TryFrom<String> for KeyAccelerator {
    type Error = AcceleratorParseError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        parse_key_accelerator(&value)
    }
}

fn parse_modifier(accelerator: &str, token: &str) -> Result<Modifiers, AcceleratorParseError> {
    let token = token.trim();
    if token.is_empty() {
        return Err(AcceleratorParseError::EmptyToken(accelerator.to_string()));
    }
    let modifier = match token.to_uppercase().as_str() {
        "OPTION" | "ALT" => Modifiers::ALT,
        "CONTROL" | "CTRL" => Modifiers::CONTROL,
        "COMMAND" | "CMD" | "SUPER" => Modifiers::META,
        "SHIFT" => Modifiers::SHIFT,
        #[cfg(target_os = "macos")]
        "COMMANDORCONTROL" | "COMMANDORCTRL" | "CMDORCTRL" | "CMDORCONTROL" => Modifiers::SUPER,
        #[cfg(not(target_os = "macos"))]
        "COMMANDORCONTROL" | "COMMANDORCTRL" | "CMDORCTRL" | "CMDORCONTROL" => Modifiers::CONTROL,
        _ => {
            return Err(AcceleratorParseError::InvalidFormat(
                accelerator.to_string(),
            ));
        }
    };
    Ok(modifier)
}

fn parse_modifiers(
    accelerator: &str,
    modifiers_str: &str,
) -> Result<Modifiers, AcceleratorParseError> {
    let mut modifiers = Modifiers::empty();
    if !modifiers_str.is_empty() {
        for token in modifiers_str.split('+') {
            modifiers |= parse_modifier(accelerator, token)?;
        }
    }

    Ok(modifiers)
}

// Tries to parse a key string as a [`Code`] first (reusing the existing parser),
// then falls back to treating it as a literal character [`Key`].
fn parse_key(key: &str) -> Result<Key, AcceleratorParseError> {
    if let Ok(code) = parse_code(key) {
        let key = code_to_key(code);
        if key != Key::Unidentified {
            return Ok(key);
        }
    }

    let trimmed = key.trim();
    if trimmed.is_empty() {
        Err(AcceleratorParseError::UnsupportedKey(key.to_string()))
    } else if parse_modifier(key, key).is_ok() {
        // The key must not be modifier
        Err(AcceleratorParseError::UnsupportedKey(key.into()))
    } else {
        Ok(Key::Character(trimmed.into()))
    }
}

fn split_key_and_modifiers(accelerator: &str) -> Result<(&str, &str), AcceleratorParseError> {
    let accelerator = accelerator.trim();
    if accelerator.is_empty() {
        return Err(AcceleratorParseError::InvalidFormat(String::new()));
    }

    // Separate modifier part from key part using rfind('+').
    // This correctly handles '+' as the key: "Ctrl++" -> rfind gives the last '+',
    // leaving "Ctrl+" as the modifier part and "" as raw key -> key is '+'.
    let (modifiers_str, key_str) = match accelerator.rfind('+') {
        Some(pos) => {
            let raw_key = &accelerator[pos + 1..];
            if raw_key.trim().is_empty() {
                // The key is '+' itself; strip the trailing separator '+' from the modifier part
                let raw_mods = accelerator[..pos].trim_end_matches('+');
                (raw_mods, "+")
            } else {
                (&accelerator[..pos], raw_key.trim())
            }
        }
        None => ("", accelerator),
    };

    Ok((modifiers_str, key_str))
}

fn parse_key_accelerator(accelerator: &str) -> Result<KeyAccelerator, AcceleratorParseError> {
    let (modifiers_str, key_str) = split_key_and_modifiers(accelerator)?;
    let mods = parse_modifiers(accelerator, modifiers_str)?;
    let key = parse_key(key_str)?;
    Ok(KeyAccelerator::new(Some(mods), key))
}

#[test]
fn test_parse_accelerator() {
    macro_rules! assert_parse_accelerator {
        ($key:literal, $lrh:expr) => {
            let r = parse_accelerator($key).unwrap();
            let l = $lrh;
            assert_eq!(r.mods, l.mods);
            assert_eq!(r.key, l.key);
        };
    }

    assert_parse_accelerator!(
        "KeyX",
        Accelerator {
            mods: Modifiers::empty(),
            key: Code::KeyX,
            id: 0,
        }
    );

    assert_parse_accelerator!(
        "CTRL+KeyX",
        Accelerator {
            mods: Modifiers::CONTROL,
            key: Code::KeyX,
            id: 0,
        }
    );

    assert_parse_accelerator!(
        "SHIFT+KeyC",
        Accelerator {
            mods: Modifiers::SHIFT,
            key: Code::KeyC,
            id: 0,
        }
    );

    assert_parse_accelerator!(
        "SHIFT+KeyC",
        Accelerator {
            mods: Modifiers::SHIFT,
            key: Code::KeyC,
            id: 0,
        }
    );

    assert_parse_accelerator!(
        "super+ctrl+SHIFT+alt+ArrowUp",
        Accelerator {
            mods: Modifiers::SUPER | Modifiers::CONTROL | Modifiers::SHIFT | Modifiers::ALT,
            key: Code::ArrowUp,
            id: 0,
        }
    );
    assert_parse_accelerator!(
        "Digit5",
        Accelerator {
            mods: Modifiers::empty(),
            key: Code::Digit5,
            id: 0,
        }
    );
    assert_parse_accelerator!(
        "KeyG",
        Accelerator {
            mods: Modifiers::empty(),
            key: Code::KeyG,
            id: 0,
        }
    );

    assert_parse_accelerator!(
        "SHiFT+F12",
        Accelerator {
            mods: Modifiers::SHIFT,
            key: Code::F12,
            id: 0,
        }
    );

    assert_parse_accelerator!(
        "CmdOrCtrl+Space",
        Accelerator {
            #[cfg(target_os = "macos")]
            mods: Modifiers::SUPER,
            #[cfg(not(target_os = "macos"))]
            mods: Modifiers::CONTROL,
            key: Code::Space,
            id: 0,
        }
    );
}

#[test]
fn test_parse_accelerator_error() {
    let cases = [
        (
            "Ctrl+Shift+C+A",
            AcceleratorParseError::InvalidFormat("Ctrl+Shift+C+A".into()),
        ),
        (
            "Ctrl+C+Shift",
            AcceleratorParseError::InvalidFormat("Ctrl+C+Shift".into()),
        ),
        ("Alt", AcceleratorParseError::UnsupportedKey("Alt".into())),
        ("Cmd", AcceleratorParseError::UnsupportedKey("Cmd".into())),
        ("Ctrl", AcceleratorParseError::UnsupportedKey("Ctrl".into())),
        (
            "Super",
            AcceleratorParseError::UnsupportedKey("Super".into()),
        ),
        ("+", AcceleratorParseError::UnsupportedKey("+".into())),
    ];
    for (text, err) in cases {
        let parsed = text.parse::<Accelerator>();
        assert_eq!(parsed, Err(err), "Expected parsing \"{text}\" to error!");
    }
}

#[test]
fn test_parse_key_accelerator_error() {
    let cases = [
        (
            "Ctrl+Shift+C+A",
            AcceleratorParseError::InvalidFormat("Ctrl+Shift+C+A".into()),
        ),
        (
            "Ctrl+C+Shift",
            AcceleratorParseError::InvalidFormat("Ctrl+C+Shift".into()),
        ),
        ("Alt", AcceleratorParseError::UnsupportedKey("Alt".into())),
        ("Cmd", AcceleratorParseError::UnsupportedKey("Cmd".into())),
        ("Ctrl", AcceleratorParseError::UnsupportedKey("Ctrl".into())),
        (
            "Super",
            AcceleratorParseError::UnsupportedKey("Super".into()),
        ),
    ];
    for (text, err) in cases {
        let parsed = text.parse::<KeyAccelerator>();
        assert_eq!(parsed, Err(err), "Expected parsing \"{text}\" to error!");
    }
}

#[test]
fn test_equality() {
    let h1 = parse_accelerator("Shift+KeyR").unwrap();
    let h2 = parse_accelerator("Shift+KeyR").unwrap();
    let h3 = Accelerator::new(Some(Modifiers::SHIFT), Code::KeyR);
    let h4 = parse_accelerator("Alt+KeyR").unwrap();
    let h5 = parse_accelerator("Alt+KeyR").unwrap();
    let h6 = parse_accelerator("KeyR").unwrap();

    assert!(h1 == h2 && h2 == h3 && h3 != h4 && h4 == h5 && h5 != h6);
    assert!(
        h1.id() == h2.id()
            && h2.id() == h3.id()
            && h3.id() != h4.id()
            && h4.id() == h5.id()
            && h5.id() != h6.id()
    );
}

#[test]
fn test_parse_key_accelerator() {
    // Basic key parsing (reuses Code-based keys)
    let cases = [
        ("Ctrl+A", Modifiers::CONTROL, Key::Character("a".into())),
        ("Ctrl+KeyA", Modifiers::CONTROL, Key::Character("a".into())),
        ("Shift+F12", Modifiers::SHIFT, Key::F12),
        // Literal '+' as key
        ("Ctrl++", Modifiers::CONTROL, Key::Character("+".into())),
        // Multiple modifiers + '+'
        (
            "Ctrl+Shift++",
            Modifiers::CONTROL | Modifiers::SHIFT,
            Key::Character("+".into()),
        ),
        // Just '+' alone
        ("+", Modifiers::empty(), Key::Character("+".into())),
        // Unicode character keys
        ("Ctrl+€", Modifiers::CONTROL, Key::Character("€".into())),
        // CmdOrCtrl works
        #[cfg(target_os = "macos")]
        (
            "CmdOrCtrl+Space",
            Modifiers::SUPER,
            Key::Character(" ".into()),
        ),
        #[cfg(not(target_os = "macos"))]
        (
            "CmdOrCtrl+Space",
            Modifiers::CONTROL,
            Key::Character(" ".into()),
        ),
    ];

    for (string, modifiers, key) in cases {
        let accelerator: KeyAccelerator = string.parse().expect("Failed to parse KeyAccelerator");
        assert_eq!(
            accelerator.mods, modifiers,
            "Expected \"{string}\" to produce modifiers: {modifiers:?}"
        );
        assert_eq!(
            accelerator.key, key,
            "Expected \"{string}\" to produce key: {key}"
        );
    }
}

#[test]
fn test_key_accelerator_from_accelerator() {
    let cases = [
        (
            Accelerator::new(Some(Modifiers::CONTROL), Code::KeyC),
            Key::Character("c".into()),
        ),
        (
            Accelerator::new(Some(Modifiers::SHIFT), Code::ArrowUp),
            Key::ArrowUp,
        ),
    ];

    for (accelerator, key) in cases {
        let key_accelerator: KeyAccelerator = accelerator.into();
        assert_eq!(
            key_accelerator.mods, accelerator.mods,
            "Source accelerator: {accelerator:?}"
        );
        assert_eq!(
            key_accelerator.key, key,
            "Source accelerator: {accelerator:?}"
        );
    }
}

#[test]
fn test_key_accelerator_equality() {
    let h1: KeyAccelerator = "Shift+R".parse().unwrap();
    let h2: KeyAccelerator = "Shift+R".parse().unwrap();
    let h3 = KeyAccelerator::new(Some(Modifiers::SHIFT), Key::Character("r".into()));
    let h4: KeyAccelerator = "Alt+R".parse().unwrap();

    assert!(h1 == h2 && h2 == h3 && h3 != h4);
    assert!(h1.id() == h2.id() && h2.id() == h3.id() && h3.id() != h4.id());

    // Converted from Accelerator should match parsed KeyAccelerator
    let from_code = Accelerator::new(Some(Modifiers::SHIFT), Code::KeyR);
    let from_code: KeyAccelerator = from_code.into();
    assert_eq!(from_code, h1);
    assert_eq!(from_code.id(), h1.id());
}
