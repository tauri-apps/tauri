use std::time::Duration;
use std::fs::File;
use std::collections::HashMap;
use attohttpc::{RequestBuilder, Method};
use http::header::HeaderName;
use serde_json::Value;
use serde::Deserialize;
use serde_repr::{Serialize_repr, Deserialize_repr};

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
  Auto
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
  Binary
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
  pub params: Option<HashMap<String, Value>>,
  /// The request headers
  pub headers: Option<HashMap<String, Value>>,
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

/// Executes an HTTP request
///
/// The response will be transformed to String,
/// If reading the response as binary, the byte array will be serialized using serde_json
pub fn make_request(options: HttpRequestOptions) -> crate::Result<String> {
  let method = Method::from_bytes(options.method.to_uppercase().as_bytes())?;
  let mut builder = RequestBuilder::new(method, options.url);
  if let Some(params) = options.params {
    for (param, param_value) in params.iter() {
      builder = builder.param(param, param_value);
    }
  }

  if let Some(headers) = options.headers {
    for (header, header_value) in headers.iter() {
      builder = builder.header(HeaderName::from_bytes(header.as_bytes())?, format!("{}", header_value));
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
  builder = builder.danger_accept_invalid_certs(true).danger_accept_invalid_hostnames(true);

  let response = if let Some(body) = options.body {
    match options.body_type.unwrap_or(BodyType::Auto) {
      BodyType::Form => builder.form(&body)?.send(),
      BodyType::File => {
        if let Some(path) = body.as_str() {
          builder.file(File::open(path)?).send()
        } else {
          return Err(crate::Error::from("Body must be the path to the file"));
        }
      },
      BodyType::Auto => {
        if body.is_object() {
          builder.json(&body)?.send()
        } else if let Some(text) = body.as_str() {
          builder.text(&text).send()
        } else if body.is_array() {
          let u: Result<Vec<u8>, _> = serde_json::from_value(body.clone());
          match u {
            Ok(vec) => builder.bytes(&vec).send(),
            Err(_) => builder.json(&body)?.send()
          }
        } else {
          builder.send()
        }
      }
    }
  } else { builder.send() };

  let response = response?;
  if response.is_success() {
    let response_data = match options.response_type.unwrap_or(ResponseType::Json) {
      ResponseType::Json => response.json()?,
      ResponseType::Text => response.text()?,
      ResponseType::Binary => serde_json::to_string(&response.bytes()?)?
    };
    Ok(response_data)
  } else {
    Err(crate::Error::from(response.status().as_str()))
  }
}