// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use http::{header::HeaderName, Method};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};

use std::{collections::HashMap, path::PathBuf, time::Duration};

/// Client builder.
#[derive(Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientBuilder {
  /// Max number of redirections to follow
  pub max_redirections: Option<usize>,
  /// Connect timeout in seconds for the request
  pub connect_timeout: Option<u64>,
}

impl ClientBuilder {
  /// Creates a new client builder with the default options.
  pub fn new() -> Self {
    Default::default()
  }

  /// Sets the maximum number of redirections.
  pub fn max_redirections(mut self, max_redirections: usize) -> Self {
    self.max_redirections = Some(max_redirections);
    self
  }

  /// Sets the connection timeout.
  pub fn connect_timeout(mut self, connect_timeout: u64) -> Self {
    self.connect_timeout = Some(connect_timeout);
    self
  }

  /// Builds the Client.
  #[cfg(not(feature = "reqwest-client"))]
  pub fn build(self) -> crate::api::Result<Client> {
    Ok(Client(self))
  }

  /// Builds the Client.
  #[cfg(feature = "reqwest-client")]
  pub fn build(self) -> crate::api::Result<Client> {
    let mut client_builder = reqwest::Client::builder();

    if let Some(max_redirections) = self.max_redirections {
      client_builder = client_builder.redirect(reqwest::redirect::Policy::limited(max_redirections))
    }

    if let Some(connect_timeout) = self.connect_timeout {
      client_builder = client_builder.connect_timeout(Duration::from_secs(connect_timeout));
    }

    let client = client_builder.build()?;
    Ok(Client(client))
  }
}

/// The HTTP client.
#[cfg(feature = "reqwest-client")]
#[derive(Clone)]
pub struct Client(reqwest::Client);

/// The HTTP client.
#[cfg(not(feature = "reqwest-client"))]
#[derive(Clone)]
pub struct Client(ClientBuilder);

#[cfg(not(feature = "reqwest-client"))]
impl Client {
  /// Executes an HTTP request
  ///
  /// The response will be transformed to String,
  /// If reading the response as binary, the byte array will be serialized using serde_json.
  pub async fn send(&self, request: HttpRequestBuilder) -> crate::api::Result<Response> {
    let method = Method::from_bytes(request.method.to_uppercase().as_bytes())?;

    let mut request_builder = attohttpc::RequestBuilder::try_new(method, &request.url)?;

    if let Some(query) = request.query {
      request_builder = request_builder.params(&query);
    }

    if let Some(headers) = request.headers {
      for (header, header_value) in headers.iter() {
        request_builder =
          request_builder.header(HeaderName::from_bytes(header.as_bytes())?, header_value);
      }
    }

    if let Some(timeout) = request.timeout {
      request_builder = request_builder.timeout(Duration::from_secs(timeout));
    }

    let response = if let Some(body) = request.body {
      match body {
        Body::Bytes(data) => request_builder.body(attohttpc::body::Bytes(data)).send()?,
        Body::Text(text) => request_builder.body(attohttpc::body::Bytes(text)).send()?,
        Body::Json(json) => request_builder.json(&json)?.send()?,
        Body::Form(form_body) => {
          let mut form = Vec::new();
          for (name, part) in form_body.0 {
            match part {
              FormPart::Bytes(bytes) => form.push((name, serde_json::to_string(&bytes)?)),
              FormPart::File(file_path) => form.push((name, serde_json::to_string(&file_path)?)),
              FormPart::Text(text) => form.push((name, text)),
            }
          }
          request_builder.form(&form)?.send()?
        }
      }
    } else {
      request_builder.send()?
    };

    Ok(Response(
      request.response_type.unwrap_or(ResponseType::Json),
      response,
      request.url,
    ))
  }
}

#[cfg(feature = "reqwest-client")]
impl Client {
  /// Executes an HTTP request
  ///
  /// The response will be transformed to String,
  /// If reading the response as binary, the byte array will be serialized using serde_json.
  pub async fn send(&self, request: HttpRequestBuilder) -> crate::api::Result<Response> {
    let method = Method::from_bytes(request.method.to_uppercase().as_bytes())?;

    let mut request_builder = self.0.request(method, &request.url);

    if let Some(query) = request.query {
      request_builder = request_builder.query(&query);
    }

    if let Some(timeout) = request.timeout {
      request_builder = request_builder.timeout(Duration::from_secs(timeout));
    }

    if let Some(body) = request.body {
      request_builder = match body {
        Body::Bytes(data) => request_builder.body(bytes::Bytes::from(data)),
        Body::Text(text) => request_builder.body(bytes::Bytes::from(text)),
        Body::Json(json) => request_builder.json(&json),
        Body::Form(form_body) => {
          let mut form = Vec::new();
          for (name, part) in form_body.0 {
            match part {
              FormPart::Bytes(bytes) => form.push((name, serde_json::to_string(&bytes)?)),
              FormPart::File(file_path) => form.push((name, serde_json::to_string(&file_path)?)),
              FormPart::Text(text) => form.push((name, text)),
            }
          }
          request_builder.form(&form)
        }
      };
    }

    let mut http_request = request_builder.build()?;
    if let Some(headers) = request.headers {
      for (header, value) in headers.iter() {
        http_request.headers_mut().insert(
          HeaderName::from_bytes(header.as_bytes())?,
          http::header::HeaderValue::from_str(value)?,
        );
      }
    }

    let response = self.0.execute(http_request).await?;

    Ok(Response(
      request.response_type.unwrap_or(ResponseType::Json),
      response,
    ))
  }
}

#[derive(Serialize_repr, Deserialize_repr, Clone, Debug)]
#[repr(u16)]
#[non_exhaustive]
/// The request's response type
pub enum ResponseType {
  /// Read the response as JSON
  Json = 1,
  /// Read the response as text
  Text,
  /// Read the response as binary
  Binary,
}

/// FormBody data types.
#[derive(Deserialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum FormPart {
  /// A file path value.
  File(PathBuf),
  /// A string value.
  Text(String),
  /// A byte array value.
  Bytes(Vec<u8>),
}

/// Form body definition.
#[derive(Deserialize)]
pub struct FormBody(HashMap<String, FormPart>);

impl FormBody {
  /// Creates a new form body.
  pub fn new(data: HashMap<String, FormPart>) -> Self {
    Self(data)
  }
}

/// A body for the request.
#[derive(Deserialize)]
#[serde(tag = "type", content = "payload")]
#[non_exhaustive]
pub enum Body {
  /// A multipart formdata body.
  Form(FormBody),
  /// A JSON body.
  Json(Value),
  /// A text string body.
  Text(String),
  /// A byte array body.
  Bytes(Vec<u8>),
}

/// The builder for a HTTP request.
///
/// # Examples
/// ```no_run
/// use tauri::api::http::{ HttpRequestBuilder, ResponseType, ClientBuilder };
/// async fn run() {
///   let client = ClientBuilder::new()
///     .max_redirections(3)
///     .build()
///     .unwrap();
///   let mut request_builder = HttpRequestBuilder::new("GET", "http://example.com");
///   let request = request_builder.response_type(ResponseType::Text);
///
///   if let Ok(response) = client.send(request).await {
///     println!("got response");
///   } else {
///     println!("Something Happened!");
///   }
/// }
/// ```
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpRequestBuilder {
  /// The request method (GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS, CONNECT or TRACE)
  pub method: String,
  /// The request URL
  pub url: String,
  /// The request query params
  pub query: Option<HashMap<String, String>>,
  /// The request headers
  pub headers: Option<HashMap<String, String>>,
  /// The request body
  pub body: Option<Body>,
  /// Timeout for the whole request
  pub timeout: Option<u64>,
  /// The response type (defaults to Json)
  pub response_type: Option<ResponseType>,
}

impl HttpRequestBuilder {
  /// Initializes a new instance of the HttpRequestrequest_builder.
  pub fn new(method: impl Into<String>, url: impl Into<String>) -> Self {
    Self {
      method: method.into(),
      url: url.into(),
      query: None,
      headers: None,
      body: None,
      timeout: None,
      response_type: None,
    }
  }

  /// Sets the request params.
  pub fn query(mut self, query: HashMap<String, String>) -> Self {
    self.query = Some(query);
    self
  }

  /// Sets the request headers.
  pub fn headers(mut self, headers: HashMap<String, String>) -> Self {
    self.headers = Some(headers);
    self
  }

  /// Sets the request body.
  pub fn body(mut self, body: Body) -> Self {
    self.body = Some(body);
    self
  }

  /// Sets the general request timeout.
  pub fn timeout(mut self, timeout: u64) -> Self {
    self.timeout = Some(timeout);
    self
  }

  /// Sets the type of the response. Interferes with the way we read the response.
  pub fn response_type(mut self, response_type: ResponseType) -> Self {
    self.response_type = Some(response_type);
    self
  }
}

/// The HTTP response.
#[cfg(feature = "reqwest-client")]
pub struct Response(ResponseType, reqwest::Response);
/// The HTTP response.
#[cfg(not(feature = "reqwest-client"))]
pub struct Response(ResponseType, attohttpc::Response, String);

impl Response {
  /// Reads the response as raw bytes.
  pub async fn bytes(self) -> crate::api::Result<RawResponse> {
    let status = self.1.status().as_u16();
    #[cfg(feature = "reqwest-client")]
    let data = self.1.bytes().await?.to_vec();
    #[cfg(not(feature = "reqwest-client"))]
    let data = self.1.bytes()?;
    Ok(RawResponse { status, data })
  }

  /// Reads the response and returns its info.
  pub async fn read(self) -> crate::api::Result<ResponseData> {
    #[cfg(feature = "reqwest-client")]
    let url = self.1.url().to_string();
    #[cfg(not(feature = "reqwest-client"))]
    let url = self.2;

    let mut headers = HashMap::new();
    for (name, value) in self.1.headers() {
      headers.insert(name.as_str().to_string(), value.to_str()?.to_string());
    }
    let status = self.1.status().as_u16();

    #[cfg(feature = "reqwest-client")]
    let data = match self.0 {
      ResponseType::Json => self.1.json().await?,
      ResponseType::Text => Value::String(self.1.text().await?),
      ResponseType::Binary => serde_json::to_value(&self.1.bytes().await?)?,
    };

    #[cfg(not(feature = "reqwest-client"))]
    let data = match self.0 {
      ResponseType::Json => self.1.json()?,
      ResponseType::Text => Value::String(self.1.text()?),
      ResponseType::Binary => serde_json::to_value(&self.1.bytes()?)?,
    };

    Ok(ResponseData {
      url,
      status,
      headers,
      data,
    })
  }
}

/// A response with raw bytes.
#[non_exhaustive]
pub struct RawResponse {
  /// Response status code.
  pub status: u16,
  /// Response bytes.
  pub data: Vec<u8>,
}

/// The response type.
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ResponseData {
  /// Response URL. Useful if it followed redirects.
  pub url: String,
  /// Response status code.
  pub status: u16,
  /// Response headers.
  pub headers: HashMap<String, String>,
  /// Response data.
  pub data: Value,
}
