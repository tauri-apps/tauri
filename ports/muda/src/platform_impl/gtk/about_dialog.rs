// Copyright 2022-2022 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Thread-safe AboutDialog wrapper for ksni tray support.
//!
//! This module provides an AboutDialog that can be safely called from
//! any thread (including ksni's DBus thread) by dispatching to the
//! GTK main thread.

use gtk4::prelude::GtkWindowExt;

use crate::AboutMetadata;

/// A thread-safe wrapper around the GTK AboutDialog.
///
/// This can be safely called from any thread, including the ksni DBus thread.
/// The `show()` method dispatches to the GTK main thread internally.
#[derive(Debug, Clone)]
pub struct AboutDialog {
    metadata: AboutMetadata,
}

impl AboutDialog {
    /// Creates a new AboutDialog with the given metadata.
    pub fn new(metadata: AboutMetadata) -> Self {
        Self { metadata }
    }

    /// Creates a new AboutDialog from compat metadata (for ksni use).
    ///
    /// This is useful when you have a `CompatAboutMetadata` from the compat layer.

    /// Shows the about dialog.
    ///
    /// This method is safe to call from any thread - it dispatches
    /// to the GTK main thread internally using `glib::MainContext::default().invoke()`.
    pub fn show(&self) {
        let metadata = self.metadata.clone();

        gtk4::glib::MainContext::default().invoke(move || {
            let dialog = gtk4::AboutDialog::new();

            // Set dialog properties from metadata
            if let Some(ref name) = metadata.name {
                dialog.set_program_name(Some(name));
            }

            if let Some(ref version) = metadata.version {
                dialog.set_version(Some(version));
            }

            if let Some(ref short_version) = metadata.short_version {
                // GTK4 doesn't have a separate short version, but we can include it
                // in the version string if both are set
                if metadata.version.is_some() {
                    // Version is already set, we could append short_version but
                    // GTK4's AboutDialog handles this differently than macOS
                    let _ = short_version; // Acknowledge unused on GTK4
                } else {
                    dialog.set_version(Some(short_version));
                }
            }

            if let Some(ref copyright) = metadata.copyright {
                dialog.set_copyright(Some(copyright));
            }

            if let Some(ref comments) = metadata.comments {
                dialog.set_comments(Some(comments));
            }

            if let Some(ref license) = metadata.license {
                dialog.set_license(Some(license));
            }

            if let Some(ref website) = metadata.website {
                dialog.set_website(Some(website));
            }

            if let Some(ref website_label) = metadata.website_label {
                dialog.set_website_label(website_label);
            }

            if let Some(ref authors) = metadata.authors {
                let authors_strs: Vec<&str> = authors.iter().map(|s| s.as_str()).collect();
                dialog.set_authors(&authors_strs);
            }

            // Note: credits in muda is Option<String>, not used directly in GTK4
            // The credits field is primarily for macOS
            let _ = &metadata.credits;

            // Present the dialog
            dialog.present();
        });
    }

    /// Returns a reference to the metadata.
    pub fn metadata(&self) -> &AboutMetadata {
        &self.metadata
    }
}
