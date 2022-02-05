// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::{
  header::{HeaderMap, HeaderName, HeaderValue},
  status::StatusCode,
  version::Version,
};
use std::fmt;

type Result<T> = core::result::Result<T, Box<dyn std::error::Error>>;

/// Represents an HTTP response
///
/// An HTTP response consists of a head and a potentially body.
///
/// ## Platform-specific
///
/// - **Linux:** Headers and status code cannot be changed.
///
/// # Examples
///
/// ```
/// # use tauri_runtime::http::*;
///
/// let response = ResponseBuilder::new()
///     .status(202)
///     .mimetype("text/html")
///     .body("hello!".as_bytes().to_vec())
///     .unwrap();
/// ```
///
pub struct Response {
  pub head: ResponseParts,
  pub body: Vec<u8>,
}

/// Component parts of an HTTP `Response`
///
/// The HTTP response head consists of a status, version, and a set of
/// header fields.
#[derive(Clone)]
pub struct ResponseParts {
  /// The response's status
  pub status: StatusCode,

  /// The response's version
  pub version: Version,

  /// The response's headers
  pub headers: HeaderMap<HeaderValue>,

  /// The response's mimetype type
  pub mimetype: Option<String>,
}

/// An HTTP response builder
///
/// This type can be used to construct an instance of `Response` through a
/// builder-like pattern.
#[derive(Debug)]
pub struct Builder {
  inner: Result<ResponseParts>,
}

impl Response {
  /// Creates a new blank `Response` with the body
  #[inline]
  pub fn new(body: Vec<u8>) -> Response {
    Response {
      head: ResponseParts::new(),
      body,
    }
  }

  /// Returns the `StatusCode`.
  #[inline]
  pub fn status(&self) -> StatusCode {
    self.head.status
  }

  /// Returns a reference to the mime type.
  #[inline]
  pub fn mimetype(&self) -> Option<String> {
    self.head.mimetype.clone()
  }

  /// Returns a reference to the associated version.
  #[inline]
  pub fn version(&self) -> Version {
    self.head.version
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
}

impl Default for Response {
  #[inline]
  fn default() -> Response {
    Response::new(Vec::new())
  }
}

impl fmt::Debug for Response {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Response")
      .field("status", &self.status())
      .field("version", &self.version())
      .field("headers", self.headers())
      .field("body", self.body())
      .finish()
  }
}

impl ResponseParts {
  /// Creates a new default instance of `ResponseParts`
  fn new() -> ResponseParts {
    ResponseParts {
      status: StatusCode::default(),
      version: Version::default(),
      headers: HeaderMap::default(),
      mimetype: None,
    }
  }
}

impl fmt::Debug for ResponseParts {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Parts")
      .field("status", &self.status)
      .field("version", &self.version)
      .field("headers", &self.headers)
      .finish()
  }
}

impl Builder {
  /// Creates a new default instance of `Builder` to construct either a
  /// `Head` or a `Response`.
  ///
  /// # Examples
  ///
  /// ```
  /// # use tauri_runtime::http::*;
  ///
  /// let response = ResponseBuilder::new()
  ///     .status(200)
  ///     .mimetype("text/html")
  ///     .body(Vec::new())
  ///     .unwrap();
  /// ```
  #[inline]
  pub fn new() -> Builder {
    Builder {
      inner: Ok(ResponseParts::new()),
    }
  }

  /// Set the HTTP mimetype for this response.
  #[must_use]
  pub fn mimetype(self, mimetype: &str) -> Self {
    self.and_then(move |mut head| {
      head.mimetype = Some(mimetype.to_string());
      Ok(head)
    })
  }

  /// Set the HTTP status for this response.
  #[must_use]
  pub fn status<T>(self, status: T) -> Self
  where
    StatusCode: TryFrom<T>,
    <StatusCode as TryFrom<T>>::Error: Into<crate::Error>,
  {
    self.and_then(move |mut head| {
      head.status = TryFrom::try_from(status).map_err(Into::into)?;
      Ok(head)
    })
  }

  /// Set the HTTP version for this response.
  ///
  /// This function will configure the HTTP version of the `Response` that
  /// will be returned from `Builder::build`.
  ///
  /// By default this is HTTP/1.1
  #[must_use]
  pub fn version(self, version: Version) -> Self {
    self.and_then(move |mut head| {
      head.version = version;
      Ok(head)
    })
  }

  /// Appends a header to this response builder.
  ///
  /// This function will append the provided key/value as a header to the
  /// internal `HeaderMap` being constructed. Essentially this is equivalent
  /// to calling `HeaderMap::append`.
  #[must_use]
  pub fn header<K, V>(self, key: K, value: V) -> Self
  where
    HeaderName: TryFrom<K>,
    <HeaderName as TryFrom<K>>::Error: Into<crate::Error>,
    HeaderValue: TryFrom<V>,
    <HeaderValue as TryFrom<V>>::Error: Into<crate::Error>,
  {
    self.and_then(move |mut head| {
      let name = <HeaderName as TryFrom<K>>::try_from(key).map_err(Into::into)?;
      let value = <HeaderValue as TryFrom<V>>::try_from(value).map_err(Into::into)?;
      head.headers.append(name, value);
      Ok(head)
    })
  }

  /// "Consumes" this builder, using the provided `body` to return a
  /// constructed `Response`.
  ///
  /// # Errors
  ///
  /// This function may return an error if any previously configured argument
  /// failed to parse or get converted to the internal representation. For
  /// example if an invalid `head` was specified via `header("Foo",
  /// "Bar\r\n")` the error will be returned when this function is called
  /// rather than when `header` was called.
  ///
  /// # Examples
  ///
  /// ```
  /// # use tauri_runtime::http::*;
  ///
  /// let response = ResponseBuilder::new()
  ///     .mimetype("text/html")
  ///     .body(Vec::new())
  ///     .unwrap();
  /// ```
  pub fn body(self, body: Vec<u8>) -> Result<Response> {
    self.inner.map(move |head| Response { head, body })
  }

  // private

  fn and_then<F>(self, func: F) -> Self
  where
    F: FnOnce(ResponseParts) -> Result<ResponseParts>,
  {
    Builder {
      inner: self.inner.and_then(func),
    }
  }
}

impl Default for Builder {
  #[inline]
  fn default() -> Builder {
    Builder {
      inner: Ok(ResponseParts::new()),
    }
  }
}
