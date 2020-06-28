use attohttpc::{Method, RequestBuilder};
use http::header::HeaderName;
use serde::Deserialize;
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};
use std::collections::HashMap;
use std::fs::File;
use std::time::Duration;

#[derive(Serialize_repr, Deserialize_repr, Clone, Debug)]
#[repr(u16)]
/// The request's body type
pub enum BodyType {
  /// Send request body as application/x-www-form-urlencoded
  Form = 1,
  /// Send request body (which is a path to a file) as application/octet-stream
  File,
  /// Detects the body type automatically
  /// - if the body is a byte array, send is as bytes (application/octet-stream)
  /// - if the body is an object or array, send it as JSON (application/json with UTF-8 charset)
  /// - if the body is a string, send it as text (text/plain with UTF-8 charset)
  Auto,
}

#[derive(Serialize_repr, Deserialize_repr, Clone, Debug)]
#[repr(u16)]
/// The request's response type
pub enum ResponseType {
  /// Read the response as JSON
  Json = 1,
  /// Read the response as text
  Text,
  /// Read the response as binary
  Binary,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
/// The configuration object of an HTTP request
pub struct HttpRequestOptions {
  /// The request method (GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS, CONNECT or TRACE)
  pub method: String,
  /// The request URL
  pub url: String,
  /// The request query params
  pub params: Option<HashMap<String, String>>,
  /// The request headers
  pub headers: Option<HashMap<String, String>>,
  /// The request body
  pub body: Option<Value>,
  /// Whether to follow redirects or not
  pub follow_redirects: Option<bool>,
  /// Max number of redirections to follow
  pub max_redirections: Option<u32>,
  /// Connect timeout for the request
  pub connect_timeout: Option<u64>,
  /// Read timeout for the request
  pub read_timeout: Option<u64>,
  /// Timeout for the whole request
  pub timeout: Option<u64>,
  /// Whether the request will announce that it accepts compression
  pub allow_compression: Option<bool>,
  /// The body type (defaults to Auto)
  pub body_type: Option<BodyType>,
  /// The response type (defaults to Json)
  pub response_type: Option<ResponseType>,
}

/// The builder for HttpRequestOptions.
///
/// # Examples
/// ```
/// # use tauri_api::http::{ HttpRequestBuilder, HttpRequestOptions, make_request, ResponseType };
/// let mut builder = HttpRequestBuilder::new("GET", "http://example.com");
/// let option = builder.response_type(ResponseType::Text)
///                     .follow_redirects(false)
///                     .build();
///
/// if let Ok(response) = make_request(option) {
///   println!("Response: {}", response);
/// } else {
///   println!("Something Happened!");
/// }
/// ```
pub struct HttpRequestBuilder {
  /// The request method (GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS, CONNECT or TRACE)
  pub method: String,
  /// The request URL
  pub url: String,
  /// The request query params
  pub params: Option<HashMap<String, String>>,
  /// The request headers
  pub headers: Option<HashMap<String, String>>,
  /// The request body
  pub body: Option<Value>,
  /// Whether to follow redirects or not
  pub follow_redirects: Option<bool>,
  /// Max number of redirections to follow
  pub max_redirections: Option<u32>,
  /// Connect timeout for the request
  pub connect_timeout: Option<u64>,
  /// Read timeout for the request
  pub read_timeout: Option<u64>,
  /// Timeout for the whole request
  pub timeout: Option<u64>,
  /// Whether the request will announce that it accepts compression
  pub allow_compression: Option<bool>,
  /// The body type (defaults to Auto)
  pub body_type: Option<BodyType>,
  /// The response type (defaults to Json)
  pub response_type: Option<ResponseType>,
}

impl HttpRequestBuilder {
  /// Initializes a new instance of the HttpRequestBuilder.
  pub fn new(method: impl Into<String>, url: impl Into<String>) -> Self {
    Self {
      method: method.into(),
      url: url.into(),
      params: None,
      headers: None,
      body: None,
      follow_redirects: None,
      max_redirections: None,
      connect_timeout: None,
      read_timeout: None,
      timeout: None,
      allow_compression: None,
      body_type: None,
      response_type: None,
    }
  }

  /// Sets the request params.
  pub fn params(mut self, params: HashMap<String, String>) -> Self {
    self.params = Some(params);
    self
  }

  /// Sets the request headers.
  pub fn headers(mut self, headers: HashMap<String, String>) -> Self {
    self.headers = Some(headers);
    self
  }

  /// Sets the request body.
  pub fn body(mut self, body: Value) -> Self {
    self.body = Some(body);
    self
  }

  /// Sets whether the request should follow redirects or not.
  pub fn follow_redirects(mut self, follow_redirects: bool) -> Self {
    self.follow_redirects = Some(follow_redirects);
    self
  }

  /// Sets the maximum number of redirections.
  pub fn max_redirections(mut self, max_redirections: u32) -> Self {
    self.max_redirections = Some(max_redirections);
    self
  }

  /// Sets the connection timeout.
  pub fn connect_timeout(mut self, connect_timeout: u64) -> Self {
    self.connect_timeout = Some(connect_timeout);
    self
  }

  /// Sets the read timeout.
  pub fn read_timeout(mut self, read_timeout: u64) -> Self {
    self.read_timeout = Some(read_timeout);
    self
  }

  /// Sets the general request timeout.
  pub fn timeout(mut self, timeout: u64) -> Self {
    self.timeout = Some(timeout);
    self
  }

  /// Sets whether the request allows compressed responses or not.
  pub fn allow_compression(mut self, allow_compression: bool) -> Self {
    self.allow_compression = Some(allow_compression);
    self
  }

  /// Sets the type of the request body.
  pub fn body_type(mut self, body_type: BodyType) -> Self {
    self.body_type = Some(body_type);
    self
  }

  /// Sets the type of the response. Interferes with the way we read the response.
  pub fn response_type(mut self, response_type: ResponseType) -> Self {
    self.response_type = Some(response_type);
    self
  }

  /// Builds the HttpRequestOptions.
  pub fn build(self) -> HttpRequestOptions {
    HttpRequestOptions {
      method: self.method,
      url: self.url,
      params: self.params,
      headers: self.headers,
      body: self.body,
      follow_redirects: self.follow_redirects,
      max_redirections: self.max_redirections,
      connect_timeout: self.connect_timeout,
      read_timeout: self.read_timeout,
      timeout: self.timeout,
      allow_compression: self.allow_compression,
      body_type: self.body_type,
      response_type: self.response_type,
    }
  }
}

/// Executes an HTTP request
///
/// The response will be transformed to String,
/// If reading the response as binary, the byte array will be serialized using serde_json
pub fn make_request(options: HttpRequestOptions) -> crate::Result<Value> {
  let method = Method::from_bytes(options.method.to_uppercase().as_bytes())?;
  let mut builder = RequestBuilder::new(method, options.url);
  if let Some(params) = options.params {
    for (param, param_value) in params.iter() {
      builder = builder.param(param, param_value);
    }
  }

  if let Some(headers) = options.headers {
    for (header, header_value) in headers.iter() {
      builder = builder.header(HeaderName::from_bytes(header.as_bytes())?, header_value);
    }
  }

  if let Some(follow_redirects) = options.follow_redirects {
    builder = builder.follow_redirects(follow_redirects);
  }
  if let Some(max_redirections) = options.max_redirections {
    builder = builder.max_redirections(max_redirections);
  }
  if let Some(connect_timeout) = options.connect_timeout {
    builder = builder.connect_timeout(Duration::from_secs(connect_timeout));
  }
  if let Some(read_timeout) = options.read_timeout {
    builder = builder.read_timeout(Duration::from_secs(read_timeout));
  }
  if let Some(timeout) = options.timeout {
    builder = builder.timeout(Duration::from_secs(timeout));
  }
  if let Some(allow_compression) = options.allow_compression {
    builder = builder.allow_compression(allow_compression);
  }
  builder = builder
    .danger_accept_invalid_certs(true)
    .danger_accept_invalid_hostnames(true);

  let response = if let Some(body) = options.body {
    match options.body_type.unwrap_or(BodyType::Auto) {
      BodyType::Form => builder.form(&body)?.send(),
      BodyType::File => {
        if let Some(path) = body.as_str() {
          builder.file(File::open(path)?).send()
        } else {
          return Err(crate::Error::Path("Body must be the path to the file".into()).into());
        }
      }
      BodyType::Auto => {
        if body.is_object() {
          builder.json(&body)?.send()
        } else if let Some(text) = body.as_str() {
          builder.text(&text).send()
        } else if body.is_array() {
          let u: Result<Vec<u8>, _> = serde_json::from_value(body.clone());
          match u {
            Ok(vec) => builder.bytes(&vec).send(),
            Err(_) => builder.json(&body)?.send(),
          }
        } else {
          builder.send()
        }
      }
    }
  } else {
    builder.send()
  };

  let response = response?;
  if response.is_success() {
    let response_data = match options.response_type.unwrap_or(ResponseType::Json) {
      ResponseType::Json => response.json::<Value>()?,
      ResponseType::Text => Value::String(response.text()?),
      ResponseType::Binary => Value::String(serde_json::to_string(&response.bytes()?)?),
    };
    Ok(response_data)
  } else {
    Err(crate::Error::Network(response.status()).into())
  }
}
