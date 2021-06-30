// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::fmt;

const MIMETYPE_PLAIN: &str = "text/plain";

/// [Web Compatible MimeTypes](https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types#important_mime_types_for_web_developers)
pub(crate) enum MimeType {
  Css,
  Csv,
  Html,
  Ico,
  Js,
  Json,
  Jsonld,
  OctetStream,
  Rtf,
  Svg,
}

impl std::fmt::Display for MimeType {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let mime = match self {
      MimeType::Css => "text/css",
      MimeType::Csv => "text/csv",
      MimeType::Html => "text/html",
      MimeType::Ico => "image/vnd.microsoft.icon",
      MimeType::Js => "text/javascript",
      MimeType::Json => "application/json",
      MimeType::Jsonld => "application/ld+json",
      MimeType::OctetStream => "application/octet-stream",
      MimeType::Rtf => "application/rtf",
      MimeType::Svg => "image/svg+xml",
    };
    write!(f, "{}", mime)
  }
}

impl MimeType {
  /// parse a URI suffix to convert text/plain mimeType to their actual web compatible mimeType.
  pub fn parse_from_uri(uri: &str) -> MimeType {
    let suffix = uri.split('.').last();
    match suffix {
      Some("bin") => Self::OctetStream,
      Some("css") => Self::Css,
      Some("csv") => Self::Csv,
      Some("html") => Self::Html,
      Some("ico") => Self::Ico,
      Some("js") => Self::Js,
      Some("json") => Self::Json,
      Some("jsonld") => Self::Jsonld,
      Some("rtf") => Self::Rtf,
      Some("svg") => Self::Svg,
      // Assume HTML when a TLD is found for eg. `wry:://tauri.studio` | `wry://hello.com`
      Some(_) => Self::Html,
      // https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types/Common_types
      // using octet stream according to this:
      None => Self::OctetStream,
    }
  }

  /// infer mimetype from content (or) URI if needed.
  pub fn parse(content: &[u8], uri: &str) -> String {
    let mime = if uri.ends_with(".svg") {
      // when reading svg, we can't use `infer`
      None
    } else {
      infer::get(content).map(|info| info.mime_type())
    };

    match mime {
      Some(mime) if mime == MIMETYPE_PLAIN => Self::parse_from_uri(uri).to_string(),
      None => Self::parse_from_uri(uri).to_string(),
      Some(mime) => mime.to_string(),
    }
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn should_parse_mimetype_from_uri() {
    let css = MimeType::parse_from_uri(
      "https://unpkg.com/browse/bootstrap@4.1.0/dist/css/bootstrap-grid.css",
    )
    .to_string();
    assert_eq!(css, "text/css".to_string());

    let csv: String = MimeType::parse_from_uri("https://example.com/random.csv").to_string();
    assert_eq!(csv, "text/csv".to_string());

    let ico: String =
      MimeType::parse_from_uri("https://icons.duckduckgo.com/ip3/microsoft.com.ico").to_string();
    assert_eq!(ico, String::from("image/vnd.microsoft.icon"));

    let html: String = MimeType::parse_from_uri("https://tauri.studio/index.html").to_string();
    assert_eq!(html, String::from("text/html"));

    let js: String =
      MimeType::parse_from_uri("https://unpkg.com/react@17.0.1/umd/react.production.min.js")
        .to_string();
    assert_eq!(js, "text/javascript".to_string());

    let json: String =
      MimeType::parse_from_uri("https://unpkg.com/browse/react@17.0.1/build-info.json").to_string();
    assert_eq!(json, String::from("application/json"));

    let jsonld: String = MimeType::parse_from_uri("https:/example.com/hello.jsonld").to_string();
    assert_eq!(jsonld, String::from("application/ld+json"));

    let rtf: String = MimeType::parse_from_uri("https://example.com/document.rtf").to_string();
    assert_eq!(rtf, String::from("application/rtf"));

    let svg: String = MimeType::parse_from_uri("https://example.com/picture.svg").to_string();
    assert_eq!(svg, String::from("image/svg+xml"));

    let custom_scheme = MimeType::parse_from_uri("wry://tauri.studio").to_string();
    assert_eq!(custom_scheme, String::from("text/html"));
  }
}
