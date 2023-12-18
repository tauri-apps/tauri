use matchit::Router;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// TODO: the current proof of concept uses [`matchit`], but we should use a simplified
/// router router that __only__ supports static and wildcard values (without wildcard params)
pub struct WindowMatcher {
  inner: Router<String>,
}

pub struct WindowMatch {
  inner: BTreeMap<String, ResolvedCommand>,
}

pub enum ResolvedCommand {
  Deny,
  Allow(ResolvedScopes<serde_json::Value>),
}

pub enum ScopeKind<'de, T>
where
  T: Serialize + Deserialize<'de>,
{
  Allow(&'de T),
  Deny(&'de T),
}

pub struct ResolvedScopes<T>
where
  T: Serialize,
  for<'de> T: Deserialize<'de>,
{
  allow: Vec<T>,
  deny: Vec<T>,
}

impl<T> ResolvedScopes<T>
where
  T: Serialize + DeserializeOwned,
  for<'de> T: Deserialize<'de>,
{
}

pub struct Resolved {}
