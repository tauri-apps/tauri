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
pub enum BodyType {
  Form = 1,
  File,
  Auto
}

#[derive(Serialize_repr, Deserialize_repr, Clone, Debug)]
#[repr(u16)]
pub enum ResponseType {
  Json = 1,
  Text,
  Binary
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpRequestOptions {
  pub method: String,
  pub url: String,
  pub params: Option<HashMap<String, Value>>,
  pub headers: Option<HashMap<String, Value>>,
  pub body: Option<Value>,
  pub follow_redirects: Option<bool>,
  pub max_redirections: Option<u32>,
  pub connect_timeout: Option<u64>,
  pub read_timeout: Option<u64>,
  pub timeout: Option<u64>,
  pub allow_compression: Option<bool>,
  pub body_type: Option<BodyType>,
  pub response_type: Option<ResponseType>,
}

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
          builder.bytes(&serde_json::to_vec(&body)?).send()
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