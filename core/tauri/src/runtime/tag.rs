// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Working with "string-able" types.

use std::{
  fmt::{Debug, Display},
  hash::Hash,
  str::FromStr,
};

/// Represents a "string-able" type.
///
/// The type is required to be able to be represented as a string [`Display`], along with knowing
/// how to be parsed from the string representation [`FromStr`]. To make sure things stay easy to
/// debug, both the [`Tag`] and the [`FromStr::Err`] must implement [`Debug`].
///
/// [`Clone`], [`Hash`], and [`Eq`] are needed so that it can represent un-hashable types.
///
/// [`Send`] and [`Sync`] and `'static` are current requirements due to how it is sometimes sent
/// across thread boundaries, although some of those constraints may relax in the future.
///
/// The simplest type that fits all these requirements is a [`String`](std::string::String).
///
/// # Handling Errors
///
/// Because we leave it up to the type to implement [`FromStr`], if an error is returned during
/// parsing then Tauri will [`std::panic!`] with the string it failed to parse.
///
/// To avoid Tauri panicking during the application runtime, have your type be able to handle
/// unknown events and never return an error in [`FromStr`]. Then it will be up to your own code
/// to handle the unknown event.
///
/// # Example
///
/// ```
/// use std::fmt;
/// use std::str::FromStr;
///
/// #[derive(Debug, Clone, Hash, Eq, PartialEq)]
/// enum Event {
///   Foo,
///   Bar,
///   Unknown(String),
/// }
///
/// impl fmt::Display for Event {
///   fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
///     f.write_str(match self {
///       Self::Foo => "foo",
///       Self::Bar => "bar",
///       Self::Unknown(s) => &s
///     })
///   }
/// }
///
/// impl FromStr for Event {
///   type Err = std::convert::Infallible;
///
///   fn from_str(s: &str) -> Result<Self, Self::Err> {
///     Ok(match s {
///       "foo" => Self::Foo,
///       "bar" => Self::Bar,
///       other => Self::Unknown(other.to_string())
///     })
///   }
/// }
///
/// // safe to unwrap because we know it's infallible due to our FromStr implementation.
/// let event: Event = "tauri://file-drop".parse().unwrap();
///
/// // show that this event type can be represented as a Tag, a requirement for using it in Tauri.
/// fn is_file_drop(tag: impl tauri::runtime::tag::Tag) {
///   assert_eq!("tauri://file-drop", tag.to_string());
/// }
///
/// is_file_drop(event);
/// ```
pub trait Tag: Hash + Eq + FromStr + Display + Debug + Clone + Send + Sync + 'static {}

/// Automatically implement [`Tag`] for all types that fit the requirements.
impl<T, E: Debug> Tag for T where
  T: Hash + Eq + FromStr<Err = E> + Display + Debug + Clone + Send + Sync + 'static
{
}

/// Private helper to turn [`Tag`] related things into JavaScript, safely.
///
/// The main concern is string escaping, so we rely on [`serde_json`] to handle all serialization
/// of the [`Tag`] as a string. We do this instead of requiring [`serde::Serialize`] on [`Tag`]
/// because it really represents a string, not any serializable data structure.
///
/// We don't want downstream users to implement this trait so that [`Tag`]s cannot be turned into
/// invalid JavaScript - regardless of their content.
pub(crate) trait ToJavascript {
  fn to_javascript(&self) -> crate::Result<String>;
}

impl<T: Tag> ToJavascript for T {
  /// Turn any [`Tag`] into the JavaScript representation of a string.
  fn to_javascript(&self) -> crate::Result<String> {
    Ok(serde_json::to_string(&self.to_string())?)
  }
}

/// Turn any collection of [`Tag`]s into a JavaScript array of strings.
pub(crate) fn tags_to_javascript_array(tags: &[impl Tag]) -> crate::Result<String> {
  let tags: Vec<String> = tags.iter().map(ToString::to_string).collect();
  Ok(serde_json::to_string(&tags)?)
}
