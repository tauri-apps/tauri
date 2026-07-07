//! Types and functions to create [`AboutMetadata`] for the [`PredefinedMenuItem::about`](crate::PredefinedMenuItem::about) dialog.

use crate::icon::Icon;

/// Application metadata for the [`PredefinedMenuItem::about`](crate::PredefinedMenuItem::about) dialog.
#[derive(Debug, Clone, Default)]
pub struct AboutMetadata {
    /// Sets the application name.
    pub name: Option<String>,
    /// The application version.
    pub version: Option<String>,
    /// The short version, e.g. "1.0".
    ///
    /// ## Platform-specific
    ///
    /// - **Windows / Linux:** Appended to the end of `version` in parentheses.
    pub short_version: Option<String>,
    /// The authors of the application.
    ///
    /// ## Platform-specific
    ///
    /// - **macOS:** Unsupported.
    pub authors: Option<Vec<String>>,
    /// Application comments.
    ///
    /// ## Platform-specific
    ///
    /// - **macOS:** Unsupported.
    pub comments: Option<String>,
    /// The copyright of the application.
    pub copyright: Option<String>,
    /// The license of the application.
    ///
    /// ## Platform-specific
    ///
    /// - **macOS:** Unsupported.
    pub license: Option<String>,
    /// The application website.
    ///
    /// ## Platform-specific
    ///
    /// - **macOS:** Unsupported.
    pub website: Option<String>,
    /// The website label.
    ///
    /// ## Platform-specific
    ///
    /// - **macOS:** Unsupported.
    pub website_label: Option<String>,
    /// The credits.
    ///
    /// ## Platform-specific
    ///
    /// - **Windows / Linux:** Unsupported.
    pub credits: Option<String>,
    /// The application icon.
    ///
    /// ## Platform-specific
    ///
    /// - **Windows:** Unsupported.
    pub icon: Option<Icon>,
}

impl AboutMetadata {
    #[allow(unused)]
    pub(crate) fn full_version(&self) -> Option<String> {
        Some(format!(
            "{}{}",
            (self.version.as_ref())?,
            (self.short_version.as_ref())
                .map(|v| format!(" ({v})"))
                .unwrap_or_default()
        ))
    }
}

/// Creates [`AboutMetadata`] from [Cargo metadata][cargo]. The following fields are set by this function.
///
/// - [`AboutMetadata::name`] (from `CARGO_PKG_NAME`)
/// - [`AboutMetadata::version`] (from `CARGO_PKG_VERSION`)
/// - [`AboutMetadata::short_version`] (from `CARGO_PKG_VERSION_MAJOR` and `CARGO_PKG_VERSION_MINOR`)
/// - [`AboutMetadata::authors`] (from `CARGO_PKG_AUTHORS`)
/// - [`AboutMetadata::comments`] (from `CARGO_PKG_DESCRIPTION`)
/// - [`AboutMetadata::license`] (from `CARGO_PKG_LICENSE`)
/// - [`AboutMetadata::website`] (from `CARGO_PKG_HOMEPAGE`)
///
/// [cargo]: https://doc.rust-lang.org/cargo/reference/environment-variables.html#environment-variables-cargo-sets-for-crates
#[macro_export]
#[doc(hidden)]
macro_rules! from_cargo_metadata {
    () => {{
        #[allow(unused_mut)]
        let mut m = $crate::about_metadata::AboutMetadata {
            name: Some(::std::env!("CARGO_PKG_NAME").into()),
            version: Some(::std::env!("CARGO_PKG_VERSION").into()),
            short_version: Some(::std::format!(
                "{}.{}",
                env!("CARGO_PKG_VERSION_MAJOR"),
                env!("CARGO_PKG_VERSION_MINOR"),
            )),
            ..::std::default::Default::default()
        };

        #[cfg(not(target_os = "macos"))]
        {
            let authors = env!("CARGO_PKG_AUTHORS")
                .split(':')
                .map(|a| a.trim().to_string())
                .collect::<::std::vec::Vec<_>>();

            m.authors = if !authors.is_empty() {
                Some(authors)
            } else {
                None
            };

            #[inline]
            fn non_empty(s: &str) -> Option<String> {
                if !s.is_empty() {
                    Some(s.to_string())
                } else {
                    None
                }
            }

            m.comments = non_empty(::std::env!("CARGO_PKG_DESCRIPTION"));
            m.license = non_empty(::std::env!("CARGO_PKG_LICENSE"));
            m.website = non_empty(::std::env!("CARGO_PKG_HOMEPAGE"));
        }

        m
    }};
}

pub use from_cargo_metadata;

/// A builder type for [`AboutMetadata`].
#[derive(Clone, Debug, Default)]
pub struct AboutMetadataBuilder(AboutMetadata);

impl AboutMetadataBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    /// Sets the application name.
    pub fn name<S: Into<String>>(mut self, name: Option<S>) -> Self {
        self.0.name = name.map(|s| s.into());
        self
    }
    /// Sets the application version.
    pub fn version<S: Into<String>>(mut self, version: Option<S>) -> Self {
        self.0.version = version.map(|s| s.into());
        self
    }
    /// Sets the short version, e.g. "1.0".
    ///
    /// ## Platform-specific
    ///
    /// - **Windows / Linux:** Appended to the end of `version` in parentheses.
    pub fn short_version<S: Into<String>>(mut self, short_version: Option<S>) -> Self {
        self.0.short_version = short_version.map(|s| s.into());
        self
    }
    /// Sets the authors of the application.
    ///
    /// ## Platform-specific
    ///
    /// - **macOS:** Unsupported.
    pub fn authors(mut self, authors: Option<Vec<String>>) -> Self {
        self.0.authors = authors;
        self
    }
    /// Application comments.
    ///
    /// ## Platform-specific
    ///
    /// - **macOS:** Unsupported.
    pub fn comments<S: Into<String>>(mut self, comments: Option<S>) -> Self {
        self.0.comments = comments.map(|s| s.into());
        self
    }
    /// Sets the copyright of the application.
    pub fn copyright<S: Into<String>>(mut self, copyright: Option<S>) -> Self {
        self.0.copyright = copyright.map(|s| s.into());
        self
    }
    /// Sets the license of the application.
    ///
    /// ## Platform-specific
    ///
    /// - **macOS:** Unsupported.
    pub fn license<S: Into<String>>(mut self, license: Option<S>) -> Self {
        self.0.license = license.map(|s| s.into());
        self
    }
    /// Sets the application website.
    ///
    /// ## Platform-specific
    ///
    /// - **macOS:** Unsupported.
    pub fn website<S: Into<String>>(mut self, website: Option<S>) -> Self {
        self.0.website = website.map(|s| s.into());
        self
    }
    /// Sets the website label.
    ///
    /// ## Platform-specific
    ///
    /// - **macOS:** Unsupported.
    pub fn website_label<S: Into<String>>(mut self, website_label: Option<S>) -> Self {
        self.0.website_label = website_label.map(|s| s.into());
        self
    }
    /// Sets the credits.
    ///
    /// ## Platform-specific
    ///
    /// - **Windows / Linux:** Unsupported.
    pub fn credits<S: Into<String>>(mut self, credits: Option<S>) -> Self {
        self.0.credits = credits.map(|s| s.into());
        self
    }
    /// Sets the application icon.
    ///
    /// ## Platform-specific
    ///
    /// - **Windows:** Unsupported.
    pub fn icon(mut self, icon: Option<Icon>) -> Self {
        self.0.icon = icon;
        self
    }

    /// Construct the final [`AboutMetadata`]
    pub fn build(self) -> AboutMetadata {
        self.0
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_build_from_metadata() {
        let m = from_cargo_metadata!();
        assert_eq!(m.name, Some("muda".to_string()));
        assert!(m.version.is_some());
        assert!(m.short_version.is_some());

        #[cfg(not(target_os = "macos"))]
        {
            assert!(matches!(m.authors, Some(a) if !a.is_empty()));
            assert!(m.comments.is_some());
            assert!(m.license.is_some());
            // Note: `m.website` is not tested because this package doesn't have the "website" field
        }
    }
}
