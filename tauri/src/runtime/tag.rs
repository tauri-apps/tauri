//! Working with "string-able" types.

use std::fmt::{Debug, Display};
use std::hash::Hash;
use std::str::FromStr;

/// Represents a "string-able" type.
///
/// The type is required to be able to be represented as a string [`Display`], along with knowing
/// how to be parsed from the string representation [`FromStr`].
///
/// [`Clone`], [`Hash`], and [`Eq`] are needed so that it can represent un-hashable types.
///
/// [`Send`] and [`Sync`] and `'static` are current requirements due to how it is sometimes sent
/// across thread boundaries, although some of those constraints may relax in the future.
///
/// The simplest type that fits all these requirements is a [`String`](std::string::String).
pub trait Tag: Hash + Eq + FromStr + Display + Debug + Clone + Send + Sync + 'static {}

/// Automatically implement [`Tag`] for all types that fit the requirements.
impl<T> Tag for T where T: Hash + Eq + FromStr + Display + Debug + Clone + Send + Sync + 'static {}

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
  let tags: Vec<String> = tags.into_iter().map(ToString::to_string).collect();
  Ok(serde_json::to_string(&tags)?)
}
