// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::fmt;

use super::{
  header::{HeaderMap, HeaderValue},
  method::Method,
};

/// Represents an HTTP request from the WebView.
///
/// An HTTP request consists of a head and a potentially optional body.
///
/// ## Platform-specific
///
/// - **Linux:** Headers are not exposed.
pub struct Request {
  pub head: RequestParts,
  pub body: Vec<u8>,
}

/// Component parts of an HTTP `Request`
///
/// The HTTP request head consists of a method, uri, and a set of
/// header fields.
#[derive(Clone)]
pub struct RequestParts {
  /// The request's method
  pub method: Method,

  /// The request's URI
  pub uri: String,

  /// The request's headers
  pub headers: HeaderMap<HeaderValue>,
}

impl Request {
  /// Creates a new blank `Request` with the body
  #[inline]
  pub fn new(body: Vec<u8>) -> Request {
    Request {
      head: RequestParts::new(),
      body,
    }
  }

  /// Returns a reference to the associated HTTP method.
  #[inline]
  pub fn method(&self) -> &Method {
    &self.head.method
  }

  /// Returns a reference to the associated URI.
  #[inline]
  pub fn uri(&self) -> &str {
    &self.head.uri
  }

  /// Returns a reference to the associated header field map.
  #[inline]
  pub fn headers(&self) -> &HeaderMap<HeaderValue> {
    &self.head.headers
  }

  /// Returns a reference to the associated HTTP body.
  #[inline]
  pub fn body(&self) -> &Vec<u8> {
    &self.body
  }

  /// Consumes the request returning the head and body RequestParts.
  #[inline]
  pub fn into_parts(self) -> (RequestParts, Vec<u8>) {
    (self.head, self.body)
  }
}

impl Default for Request {
  fn default() -> Request {
    Request::new(Vec::new())
  }
}

impl fmt::Debug for Request {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Request")
      .field("method", self.method())
      .field("uri", &self.uri())
      .field("headers", self.headers())
      .field("body", self.body())
      .finish()
  }
}

impl RequestParts {
  /// Creates a new default instance of `RequestParts`
  fn new() -> RequestParts {
    RequestParts {
      method: Method::default(),
      uri: "".into(),
      headers: HeaderMap::default(),
    }
  }
}

impl fmt::Debug for RequestParts {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Parts")
      .field("method", &self.method)
      .field("uri", &self.uri)
      .field("headers", &self.headers)
      .finish()
  }
}
