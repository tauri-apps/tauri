// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use keyboard_types::{Key, Modifiers};

use crate::accelerator::KeyAccelerator;

impl KeyAccelerator {
    /// Render as a GTK accelerator string, e.g. `<Control><Shift>c`.
    ///
    /// The result is parseable by `gtk::accelerator_parse` and accepted by
    /// `gio::prelude::ActionMapExt::set_accels_for_action`.
    pub fn to_gtk(&self) -> String {
        let mut gtk = modifiers_to_gtk(self.modifiers());
        gtk.push_str(&key_to_gtk(self.key()));
        gtk
    }
}

fn modifiers_to_gtk(mods: Modifiers) -> String {
    let mut gtk = String::new();

    if mods.contains(Modifiers::SHIFT) {
        gtk.push_str("<Shift>");
    }
    if mods.contains(Modifiers::CONTROL) {
        gtk.push_str("<Control>");
    }
    if mods.contains(Modifiers::ALT) {
        gtk.push_str("<Alt>");
    }
    if mods.contains(Modifiers::META) {
        gtk.push_str("<Meta>");
    }
    gtk
}

fn key_to_gtk(key: &Key) -> String {
    match key {
        // A single character maps straight to its GDK key name, except
        // space, which GDK names rather than prints.
        Key::Character(c) if c == " " => "space".to_string(),
        Key::Character(c) => c.to_lowercase(),
        // Named keys whose GDK name differs from the W3C name.
        Key::Enter => "Return".to_string(),
        Key::ArrowUp => "Up".to_string(),
        Key::ArrowDown => "Down".to_string(),
        Key::ArrowLeft => "Left".to_string(),
        Key::ArrowRight => "Right".to_string(),
        Key::Backspace => "BackSpace".to_string(),
        Key::PageUp => "Page_Up".to_string(),
        Key::PageDown => "Page_Down".to_string(),
        // Escape, Delete, Tab, Home, End, Insert, F-keys, and space
        // already match the GDK name that `accelerator_parse` understands.
        other => other.to_string(),
    }
}
