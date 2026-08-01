// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Host-preserving mapped custom-protocol routing for CEF.
//!
//! CEF — like WebView2 and Android WebView — cannot expose a non-standard
//! custom scheme to `fetch`/storage/etc. without registering that scheme in
//! every process. Tauri instead maps a logical custom-protocol URL onto the
//! built-in `https` (or `http`) scheme, exactly like Wry's WebView2/Android
//! workaround (`wry::custom_protocol_workaround`):
//!
//! ```text
//! logical   scheme://authority/path
//! mapped    https://scheme.authority/path
//! ```
//!
//! The `scheme.` label is prepended to the authority, so the *logical authority
//! is preserved* as a browser-visible subdomain rather than collapsed to a
//! single fixed host. That is what lets distinct logical authorities on one
//! scheme (`app://a.localhost/…`, `app://b.localhost/…`) become distinct
//! browser origins (`https://app.a.localhost/…`, `https://app.b.localhost/…`),
//! each with its own same-origin storage.
//!
//! Unlike Wry — whose per-protocol interception filter is scoped by the host
//! runtime — the CEF factory is registered against the whole built-in scheme
//! (see [`crate::cef_impl::request_context`]). Routing therefore has to do two
//! extra jobs here:
//!
//! 1. [`split_mapped_host`] restricts interception to the reserved
//!    `<scheme>.….localhost` namespace, so unrelated HTTPS is never claimed.
//! 2. [`to_logical_url`] normalizes an intercepted mapped URL back to the
//!    logical custom-protocol request before it reaches the Tauri handler, so
//!    the handler always receives one logical URL shape across every runtime.

use url::Url;

#[derive(Debug, PartialEq, Eq)]
pub(crate) struct MappedRequest {
  pub(crate) custom_scheme: String,
  pub(crate) logical_url: String,
}

/// Splits a mapped work-around host `<scheme>.<authority>` into its logical
/// scheme label and logical authority, but only for the reserved
/// `*.localhost` namespace.
///
/// Returns `None` for any host that is not `<non-empty-label>.….localhost`, so
/// a factory registered against the whole `https`/`http` scheme leaves
/// unrelated origins untouched.
///
/// ```text
/// app.localhost                -> ("app", "localhost")
/// app.a.localhost              -> ("app", "a.localhost")
/// localhost | example.com      -> None
/// app.a.localhost.example.com  -> None  (does not end in .localhost)
/// ```
pub(crate) fn split_mapped_host(host: &str) -> Option<(&str, &str)> {
  // A bare `localhost` is not part of the reserved namespace: the scheme label
  // is mandatory, so the host must have at least `<label>.localhost`.
  if !host.ends_with(".localhost") {
    return None;
  }

  let (label, authority) = host.split_once('.')?;
  if label.is_empty()
    || authority.is_empty()
    || label.split('.').any(str::is_empty)
    || authority.split('.').any(str::is_empty)
  {
    return None;
  }

  Some((label, authority))
}

/// Reverts a mapped browser URL back to its logical custom-protocol form,
/// mirroring `wry::custom_protocol_workaround::revert_uri_work_around`.
///
/// `mapped_scheme` is the built-in scheme the logical scheme was mapped onto
/// (`https` or `http`); `scheme` is the logical custom scheme. Returns `None`
/// when `url` is not a mapped URL for `scheme`. Only the scheme + first host
/// label are rewritten, so the logical authority (and everything after it) is
/// preserved verbatim.
///
/// ```text
/// https://app.a.localhost/index.html -> app://a.localhost/index.html
/// https://app.localhost/x            -> app://localhost/x
/// ```
pub(crate) fn to_logical_url(url: &str, mapped_scheme: &str, scheme: &str) -> Option<String> {
  let prefix = format!("{mapped_scheme}://{scheme}.");
  let authority_and_rest = url.strip_prefix(&prefix)?;
  Some(format!("{scheme}://{authority_and_rest}"))
}

/// Classifies one built-in-scheme request for the CEF scheme-handler factory.
///
/// A request is routed only when it uses HTTP(S), its host is in the reserved
/// mapped namespace, and the extracted custom scheme is registered for this
/// browser. Returning `None` deliberately hands the request back to CEF's
/// built-in network stack.
pub(crate) fn route_mapped_request(
  request_url: &str,
  mut is_registered: impl FnMut(&str) -> bool,
) -> Option<MappedRequest> {
  let parsed = Url::parse(request_url).ok()?;
  if !matches!(parsed.scheme(), "http" | "https") {
    return None;
  }

  let (custom_scheme, _authority) = split_mapped_host(parsed.host_str()?)?;
  if !is_registered(custom_scheme) {
    return None;
  }

  Some(MappedRequest {
    custom_scheme: custom_scheme.to_string(),
    // Use url's canonical serialization so scheme/host casing cannot make the
    // prefix reversal disagree with the parsed routing decision.
    logical_url: to_logical_url(parsed.as_str(), parsed.scheme(), custom_scheme)?,
  })
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn splits_reserved_hosts_and_preserves_authority() {
    assert_eq!(
      split_mapped_host("app.localhost"),
      Some(("app", "localhost"))
    );
    assert_eq!(
      split_mapped_host("my-scheme.localhost"),
      Some(("my-scheme", "localhost"))
    );
    // A multi-label authority (the host-preserving case) keeps everything after
    // the scheme label intact.
    assert_eq!(
      split_mapped_host("app.a1b2c3.localhost"),
      Some(("app", "a1b2c3.localhost"))
    );
  }

  #[test]
  fn rejects_hosts_outside_the_reserved_namespace() {
    // A factory registered on the whole scheme must ignore these so they fall
    // through to CEF's normal network path.
    assert_eq!(split_mapped_host("localhost"), None);
    assert_eq!(split_mapped_host("example.com"), None);
    assert_eq!(split_mapped_host("app.localhost.example.com"), None);
    assert_eq!(split_mapped_host("app.a.localhost.example.com"), None);
    assert_eq!(split_mapped_host(".localhost"), None);
    assert_eq!(split_mapped_host("app..localhost"), None);
    assert_eq!(split_mapped_host("app.a..localhost"), None);
  }

  #[test]
  fn distinct_authorities_stay_distinct_on_one_scheme() {
    let a = split_mapped_host("app.a.localhost").unwrap();
    let b = split_mapped_host("app.b.localhost").unwrap();
    assert_eq!(a.0, b.0, "same logical scheme");
    assert_ne!(a.1, b.1, "distinct logical authorities / origins");
  }

  #[test]
  fn reverts_mapped_url_to_logical_preserving_authority() {
    assert_eq!(
      to_logical_url("https://app.a.localhost/index.html", "https", "app").as_deref(),
      Some("app://a.localhost/index.html")
    );
    assert_eq!(
      to_logical_url("https://app.localhost/x", "https", "app").as_deref(),
      Some("app://localhost/x")
    );
    // The mapping also works for the `http` variant used when `use_https_scheme`
    // is disabled.
    assert_eq!(
      to_logical_url("http://app.a.localhost/", "http", "app").as_deref(),
      Some("app://a.localhost/")
    );
  }

  #[test]
  fn revert_is_none_for_urls_outside_this_scheme() {
    assert_eq!(
      to_logical_url("https://example.com/x", "https", "app"),
      None
    );
    // A different logical scheme's mapped host is not this scheme's business.
    assert_eq!(
      to_logical_url("https://other.localhost/x", "https", "app"),
      None
    );
  }

  #[test]
  fn entrypoints_under_one_authority_revert_to_one_origin() {
    for path in ["/index.html", "/headless-runtime.html"] {
      let mapped = format!("https://app.a1b2c3.localhost{path}");
      let logical = to_logical_url(&mapped, "https", "app").unwrap();
      assert_eq!(logical, format!("app://a1b2c3.localhost{path}"));
    }
  }

  #[test]
  fn routes_two_authorities_for_the_registered_handler_and_normalizes_both() {
    for authority in ["profile-one.localhost", "profile-two.localhost"] {
      let mapped = format!("https://app.{authority}/index.html");
      assert_eq!(
        route_mapped_request(&mapped, |scheme| scheme == "app"),
        Some(MappedRequest {
          custom_scheme: "app".into(),
          logical_url: format!("app://{authority}/index.html"),
        })
      );
    }
  }

  #[test]
  fn routing_rejects_malformed_unknown_and_unrelated_urls() {
    let registered = |scheme: &str| scheme == "app";

    assert_eq!(
      route_mapped_request("https://app..localhost/index.html", registered),
      None
    );
    assert_eq!(
      route_mapped_request("https://unknown.a.localhost/index.html", registered),
      None
    );
    assert_eq!(
      route_mapped_request("https://example.com/index.html", registered),
      None
    );
    assert_eq!(
      route_mapped_request("file://app.a.localhost/index.html", registered),
      None
    );
  }

  #[test]
  fn routing_normalizes_scheme_and_host_casing_before_reversal() {
    assert_eq!(
      route_mapped_request("HTTPS://APP.A.LOCALHOST/index.html", |scheme| scheme
        == "app"),
      Some(MappedRequest {
        custom_scheme: "app".into(),
        logical_url: "app://a.localhost/index.html".into(),
      })
    );
  }
}
