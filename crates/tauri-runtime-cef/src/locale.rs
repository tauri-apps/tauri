// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! The languages the user asked their operating system for, as Chromium wants them.
//!
//! CEF appends `--lang=en-US` whenever `CefSettings.locale` is empty, and with no
//! `accept_language_list` of its own the accept-language list falls back to that locale.
//! A CEF application therefore sends `Accept-Language: en-US,en` and reports
//! `navigator.language === "en-US"` for every user on earth, whatever their system is set
//! to — which silently breaks server-side localization and any page that branches on the
//! browser language.
//!
//! `CefSettings.locale` cannot simply follow the system, because it selects which
//! `locales/*.pak` Chromium loads its own localized strings from and Tauri's bundler
//! packages only `en-US.pak`. `accept_language_list` has no such constraint: it is a plain
//! list of language codes, so it is the one the runtime derives from the system.
//!
//! CEF expands whatever it is given — `pt-BR` becomes `pt-BR,pt` with quality values —
//! through `net::HttpUtil::ExpandLanguageList`, so this module only has to produce the
//! ordered list of codes the user actually chose.

/// The user's preferred languages as a comma-delimited list of BCP-47 codes, or [`None`]
/// when the system does not say.
///
/// The result feeds `CefSettings.accept_language_list`.
pub(crate) fn system_accept_language_list() -> Option<String> {
  let languages = preferred_languages();
  let list = normalize_language_list(languages.iter().map(String::as_str));
  (!list.is_empty()).then(|| list.join(","))
}

/// Cleans up the language tags a platform reports, into the spelling Chromium expects.
///
/// POSIX locale names are not BCP-47: they spell the region after an underscore and carry
/// a charset and a modifier the web has no use for (`pt_BR.UTF-8@euro`). The placeholder
/// locales are dropped rather than sent — `C` and `POSIX` mean "no preference", and
/// asking a server for a language called `c` is worse than asking for nothing.
///
/// Duplicates are dropped keeping first position, because the order is the user's
/// preference order.
fn normalize_language_list<'a>(languages: impl Iterator<Item = &'a str>) -> Vec<String> {
  let mut normalized: Vec<String> = Vec::new();

  for language in languages {
    let Some(tag) = normalize_language(language) else {
      continue;
    };
    if !normalized.iter().any(|existing| existing == &tag) {
      normalized.push(tag);
    }
  }

  normalized
}

/// Normalizes one language tag, returning [`None`] for one that must not be sent.
fn normalize_language(language: &str) -> Option<String> {
  // `pt_BR.UTF-8@euro` -> `pt_BR`
  let tag = language
    .split(['.', '@'])
    .next()
    .unwrap_or_default()
    .trim()
    .replace('_', "-");

  if tag.is_empty() || tag.eq_ignore_ascii_case("C") || tag.eq_ignore_ascii_case("POSIX") {
    return None;
  }

  // A tag is language[-Script][-REGION]; anything else is not something to send to a
  // server, and a stray value in `Accept-Language` is a fingerprinting surface.
  let mut parts = tag.split('-');
  let language = parts.next()?;
  if language.len() < 2 || language.len() > 8 || !language.chars().all(|c| c.is_ascii_alphabetic())
  {
    return None;
  }
  if !parts.all(|part| {
    !part.is_empty() && part.len() <= 8 && part.chars().all(|c| c.is_ascii_alphanumeric())
  }) {
    return None;
  }

  Some(tag)
}

/// The ordered languages the system reports, in whatever spelling it uses.
#[cfg(any(
  target_os = "linux",
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "netbsd",
  target_os = "openbsd"
))]
fn preferred_languages() -> Vec<String> {
  // `LANGUAGE` is the only one that carries a *list*, and gettext gives it priority over
  // the `LC_*` variables for exactly the "I read these languages, in this order" question
  // being asked here. The rest name a single locale, in the precedence POSIX defines.
  if let Some(languages) = std::env::var_os("LANGUAGE")
    .map(|value| value.to_string_lossy().into_owned())
    .filter(|value| !value.is_empty())
  {
    let languages: Vec<String> = languages.split(':').map(ToString::to_string).collect();
    if !normalize_language_list(languages.iter().map(String::as_str)).is_empty() {
      return languages;
    }
  }

  for variable in ["LC_ALL", "LC_MESSAGES", "LANG"] {
    if let Some(value) = std::env::var_os(variable)
      .map(|value| value.to_string_lossy().into_owned())
      .filter(|value| !value.is_empty())
    {
      return vec![value];
    }
  }

  Vec::new()
}

/// The ordered languages the system reports, in whatever spelling it uses.
#[cfg(target_os = "macos")]
fn preferred_languages() -> Vec<String> {
  // Already BCP-47, already in the order set in System Settings.
  objc2_foundation::NSLocale::preferredLanguages()
    .iter()
    .map(|language| language.to_string())
    .collect()
}

/// The ordered languages the system reports, in whatever spelling it uses.
#[cfg(windows)]
fn preferred_languages() -> Vec<String> {
  use windows::Win32::Globalization::GetUserDefaultLocaleName;

  // `LOCALE_NAME_MAX_LENGTH`, which is the documented bound on what this can write.
  let mut buffer = [0u16; 85];
  // SAFETY: the binding takes the buffer as a slice and derives the length from it.
  let written = unsafe { GetUserDefaultLocaleName(&mut buffer) };
  if written <= 0 {
    return Vec::new();
  }

  // The count includes the terminating null.
  let name = String::from_utf16_lossy(&buffer[..(written as usize).saturating_sub(1)]);
  if name.is_empty() {
    Vec::new()
  } else {
    vec![name]
  }
}

#[cfg(not(any(
  target_os = "linux",
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "netbsd",
  target_os = "openbsd",
  target_os = "macos",
  windows
)))]
fn preferred_languages() -> Vec<String> {
  Vec::new()
}

#[cfg(test)]
mod tests {
  use super::*;

  fn normalize(languages: &[&str]) -> Vec<String> {
    normalize_language_list(languages.iter().copied())
  }

  #[test]
  fn posix_locale_names_become_language_tags() {
    assert_eq!(normalize(&["pt_BR.UTF-8"]), ["pt-BR"]);
    assert_eq!(normalize(&["de_DE@euro"]), ["de-DE"]);
    assert_eq!(normalize(&["fr_CA.ISO-8859-1@currency"]), ["fr-CA"]);
  }

  #[test]
  fn tags_that_are_already_bcp47_are_left_alone() {
    assert_eq!(
      normalize(&["pt-BR", "zh-Hans-CN", "en"]),
      ["pt-BR", "zh-Hans-CN", "en"]
    );
  }

  #[test]
  fn placeholder_locales_are_dropped() {
    // "C" and "POSIX" mean "no preference"; sending them asks servers for a language
    // called "c".
    assert!(normalize(&["C"]).is_empty());
    assert!(normalize(&["POSIX"]).is_empty());
    assert!(normalize(&["C.UTF-8"]).is_empty());
    assert_eq!(normalize(&["C", "pt_BR"]), ["pt-BR"]);
  }

  #[test]
  fn preference_order_is_kept_and_duplicates_dropped() {
    assert_eq!(
      normalize(&["pt_BR.UTF-8", "pt-BR", "en_US.UTF-8"]),
      ["pt-BR", "en-US"]
    );
  }

  #[test]
  fn malformed_tags_never_reach_the_accept_language_header() {
    assert!(normalize(&[""]).is_empty());
    assert!(normalize(&["  "]).is_empty());
    assert!(
      normalize(&["e"]).is_empty(),
      "a one-letter language is not one"
    );
    assert!(
      normalize(&["en-"]).is_empty(),
      "a trailing separator is malformed"
    );
    assert!(normalize(&["en-US-"]).is_empty());
    assert!(normalize(&["1234"]).is_empty());
    assert!(
      normalize(&["en US"]).is_empty(),
      "a space is not a separator"
    );
  }

  #[test]
  fn nothing_reported_produces_no_list() {
    assert!(normalize(&[]).is_empty());
    assert!(normalize(&["C", "POSIX"]).is_empty());
  }
}
