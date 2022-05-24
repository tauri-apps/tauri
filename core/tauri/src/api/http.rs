// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Types and functions related to HTTP request.

use http::Method;
pub use http::StatusCode;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
use serde_repr::{Deserialize_repr, Serialize_repr};
use url::Url;

use std::{collections::HashMap, path::PathBuf, time::Duration};

#[cfg(feature = "reqwest-client")]
pub use reqwest::header;

#[cfg(not(feature = "reqwest-client"))]
pub use attohttpc::header;

use header::{HeaderName, HeaderValue};

#[derive(Deserialize)]
#[serde(untagged)]
enum SerdeDuration {
  Seconds(u64),
  Duration(Duration),
}

fn deserialize_duration<'de, D: Deserializer<'de>>(
  deserializer: D,
) -> Result<Option<Duration>, D::Error> {
  if let Some(duration) = Option::<SerdeDuration>::deserialize(deserializer)? {
    Ok(Some(match duration {
      SerdeDuration::Seconds(s) => Duration::from_secs(s),
      SerdeDuration::Duration(d) => d,
    }))
  } else {
    Ok(None)
  }
}

/// The builder of [`Client`].
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClientBuilder {
  /// Max number of redirections to follow.
  pub max_redirections: Option<usize>,
  /// Connect timeout for the request.
  #[serde(deserialize_with = "deserialize_duration", default)]
  pub connect_timeout: Option<Duration>,
}

impl ClientBuilder {
  /// Creates a new client builder with the default options.
  pub fn new() -> Self {
    Default::default()
  }

  /// Sets the maximum number of redirections.
  #[must_use]
  pub fn max_redirections(mut self, max_redirections: usize) -> Self {
    self.max_redirections = Some(max_redirections);
    self
  }

  /// Sets the connection timeout.
  #[must_use]
  pub fn connect_timeout(mut self, connect_timeout: Duration) -> Self {
    self.connect_timeout.replace(connect_timeout);
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
      client_builder = client_builder.connect_timeout(connect_timeout);
    }

    let client = client_builder.build()?;
    Ok(Client(client))
  }
}

/// The HTTP client based on [`reqwest`].
#[cfg(feature = "reqwest-client")]
#[derive(Debug, Clone)]
pub struct Client(reqwest::Client);

/// The HTTP client.
#[cfg(not(feature = "reqwest-client"))]
#[derive(Debug, Clone)]
pub struct Client(ClientBuilder);

#[cfg(not(feature = "reqwest-client"))]
impl Client {
  /// Executes an HTTP request.
  ///
  /// # Examples
  ///
  /// ```rust,no_run
  /// use tauri::api::http::{ClientBuilder, HttpRequestBuilder, ResponseType};
  /// async fn run_request() {
  ///   let client = ClientBuilder::new().build().unwrap();
  ///   let response = client.send(
  ///     HttpRequestBuilder::new("GET", "https://www.rust-lang.org")
  ///       .unwrap()
  ///       .response_type(ResponseType::Binary)
  ///   ).await;
  ///   if let Ok(response) = response {
  ///     let bytes = response.bytes();
  ///   }
  /// }
  /// ```
  pub async fn send(&self, request: HttpRequestBuilder) -> crate::api::Result<Response> {
    let method = Method::from_bytes(request.method.to_uppercase().as_bytes())?;

    let mut request_builder = attohttpc::RequestBuilder::try_new(method, &request.url)?;

    if let Some(query) = request.query {
      request_builder = request_builder.params(&query);
    }

    if let Some(headers) = &request.headers {
      for (name, value) in headers.0.iter() {
        request_builder = request_builder.header(name, value);
      }
    }

    if let Some(timeout) = request.timeout {
      request_builder = request_builder.timeout(timeout);
      #[cfg(windows)]
      {
        // on Windows the global timeout is not respected, see https://github.com/sbstp/attohttpc/issues/118
        request_builder = request_builder.read_timeout(timeout);
      }
    }

    let response = if let Some(body) = request.body {
      match body {
        Body::Bytes(data) => request_builder.body(attohttpc::body::Bytes(data)).send()?,
        Body::Text(text) => request_builder.body(attohttpc::body::Bytes(text)).send()?,
        Body::Json(json) => request_builder.json(&json)?.send()?,
        Body::Form(form_body) => {
          #[allow(unused_variables)]
          fn send_form(
            request_builder: attohttpc::RequestBuilder,
            headers: &Option<HeaderMap>,
            form_body: FormBody,
          ) -> crate::api::Result<attohttpc::Response> {
            #[cfg(feature = "http-multipart")]
            if matches!(
              headers
                .as_ref()
                .and_then(|h| h.0.get("content-type"))
                .map(|v| v.as_bytes()),
              Some(b"multipart/form-data")
            ) {
              let mut multipart = attohttpc::MultipartBuilder::new();
              let mut byte_cache: HashMap<String, Vec<u8>> = Default::default();

              for (name, part) in &form_body.0 {
                if let FormPart::File { file, .. } = part {
                  byte_cache.insert(name.to_string(), file.clone().try_into()?);
                }
              }
              for (name, part) in &form_body.0 {
                multipart = match part {
                  FormPart::File {
                    file,
                    mime,
                    file_name,
                  } => {
                    // safe to unwrap: always set by previous loop
                    let mut file =
                      attohttpc::MultipartFile::new(name, byte_cache.get(name).unwrap());
                    if let Some(mime) = mime {
                      file = file.with_type(mime)?;
                    }
                    if let Some(file_name) = file_name {
                      file = file.with_filename(file_name);
                    }
                    multipart.with_file(file)
                  }
                  FormPart::Text(value) => multipart.with_text(name, value),
                };
              }
              return request_builder
                .body(multipart.build()?)
                .send()
                .map_err(Into::into);
            }

            let mut form = Vec::new();
            for (name, part) in form_body.0 {
              match part {
                FormPart::File { file, .. } => {
                  let bytes: Vec<u8> = file.try_into()?;
                  form.push((name, serde_json::to_string(&bytes)?))
                }
                FormPart::Text(value) => form.push((name, value)),
              }
            }
            request_builder.form(&form)?.send().map_err(Into::into)
          }

          send_form(request_builder, &request.headers, form_body)?
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
  /// # Examples
  pub async fn send(&self, request: HttpRequestBuilder) -> crate::api::Result<Response> {
    let method = Method::from_bytes(request.method.to_uppercase().as_bytes())?;

    let mut request_builder = self.0.request(method, request.url.as_str());

    if let Some(query) = request.query {
      request_builder = request_builder.query(&query);
    }

    if let Some(timeout) = request.timeout {
      request_builder = request_builder.timeout(timeout);
    }

    if let Some(body) = request.body {
      request_builder = match body {
        Body::Bytes(data) => request_builder.body(bytes::Bytes::from(data)),
        Body::Text(text) => request_builder.body(bytes::Bytes::from(text)),
        Body::Json(json) => request_builder.json(&json),
        Body::Form(form_body) => {
          #[allow(unused_variables)]
          fn send_form(
            request_builder: reqwest::RequestBuilder,
            headers: &Option<HeaderMap>,
            form_body: FormBody,
          ) -> crate::api::Result<reqwest::RequestBuilder> {
            #[cfg(feature = "http-multipart")]
            if matches!(
              headers
                .as_ref()
                .and_then(|h| h.0.get("content-type"))
                .map(|v| v.as_bytes()),
              Some(b"multipart/form-data")
            ) {
              let mut multipart = reqwest::multipart::Form::new();

              for (name, part) in form_body.0 {
                let part = match part {
                  FormPart::File {
                    file,
                    mime,
                    file_name,
                  } => {
                    let bytes: Vec<u8> = file.try_into()?;
                    let mut part = reqwest::multipart::Part::bytes(bytes);
                    if let Some(mime) = mime {
                      part = part.mime_str(&mime)?;
                    }
                    if let Some(file_name) = file_name {
                      part = part.file_name(file_name);
                    }
                    part
                  }
                  FormPart::Text(value) => reqwest::multipart::Part::text(value),
                };

                multipart = multipart.part(name, part);
              }

              return Ok(request_builder.multipart(multipart));
            }

            let mut form = Vec::new();
            for (name, part) in form_body.0 {
              match part {
                FormPart::File { file, .. } => {
                  let bytes: Vec<u8> = file.try_into()?;
                  form.push((name, serde_json::to_string(&bytes)?))
                }
                FormPart::Text(value) => form.push((name, value)),
              }
            }
            Ok(request_builder.form(&form))
          }
          send_form(request_builder, &request.headers, form_body)?
        }
      };
    }

    if let Some(headers) = request.headers {
      request_builder = request_builder.headers(headers.0);
    }

    let http_request = request_builder.build()?;

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
/// The HTTP response type.
pub enum ResponseType {
  /// Read the response as JSON
  Json = 1,
  /// Read the response as text
  Text,
  /// Read the response as binary
  Binary,
}

/// A file path or contents.
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum FilePart {
  /// File path.
  Path(PathBuf),
  /// File contents.
  Contents(Vec<u8>),
}

impl TryFrom<FilePart> for Vec<u8> {
  type Error = crate::api::Error;
  fn try_from(file: FilePart) -> crate::api::Result<Self> {
    let bytes = match file {
      FilePart::Path(path) => std::fs::read(&path)?,
      FilePart::Contents(bytes) => bytes,
    };
    Ok(bytes)
  }
}

/// [`FormBody`] data types.
#[derive(Debug, Deserialize)]
#[serde(untagged)]
#[non_exhaustive]
pub enum FormPart {
  /// A string value.
  Text(String),
  /// A file value.
  #[serde(rename_all = "camelCase")]
  File {
    /// File path or content.
    file: FilePart,
    /// Mime type of this part.
    /// Only used when the `Content-Type` header is set to `multipart/form-data`.
    mime: Option<String>,
    /// File name.
    /// Only used when the `Content-Type` header is set to `multipart/form-data`.
    file_name: Option<String>,
  },
}

/// Form body definition.
#[derive(Debug, Deserialize)]
pub struct FormBody(pub(crate) HashMap<String, FormPart>);

impl FormBody {
  /// Creates a new form body.
  pub fn new(data: HashMap<String, FormPart>) -> Self {
    Self(data)
  }
}

/// A body for the request.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "payload")]
#[non_exhaustive]
pub enum Body {
  /// A form body.
  Form(FormBody),
  /// A JSON body.
  Json(Value),
  /// A text string body.
  Text(String),
  /// A byte array body.
  Bytes(Vec<u8>),
}

/// A set of HTTP headers.
#[derive(Debug, Default)]
pub struct HeaderMap(header::HeaderMap);

impl<'de> Deserialize<'de> for HeaderMap {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let map = HashMap::<String, String>::deserialize(deserializer)?;
    let mut headers = header::HeaderMap::default();
    for (key, value) in map {
      if let (Ok(key), Ok(value)) = (
        header::HeaderName::from_bytes(key.as_bytes()),
        header::HeaderValue::from_str(&value),
      ) {
        headers.insert(key, value);
      } else {
        return Err(serde::de::Error::custom(format!(
          "invalid header `{}` `{}`",
          key, value
        )));
      }
    }
    Ok(Self(headers))
  }
}

/// The builder for a HTTP request.
///
/// # Examples
/// ```rust,no_run
/// use tauri::api::http::{HttpRequestBuilder, ResponseType, ClientBuilder};
/// async fn run() {
///   let client = ClientBuilder::new()
///     .max_redirections(3)
///     .build()
///     .unwrap();
///   let request = HttpRequestBuilder::new("GET", "http://example.com").unwrap()
///     .response_type(ResponseType::Text);
///   if let Ok(response) = client.send(request).await {
///     println!("got response");
///   } else {
///     println!("Something Happened!");
///   }
/// }
/// ```
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpRequestBuilder {
  /// The request method (GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS, CONNECT or TRACE)
  pub method: String,
  /// The request URL
  pub url: Url,
  /// The request query params
  pub query: Option<HashMap<String, String>>,
  /// The request headers
  pub headers: Option<HeaderMap>,
  /// The request body
  pub body: Option<Body>,
  /// Timeout for the whole request
  #[serde(deserialize_with = "deserialize_duration", default)]
  pub timeout: Option<Duration>,
  /// The response type (defaults to Json)
  pub response_type: Option<ResponseType>,
}

impl HttpRequestBuilder {
  /// Initializes a new instance of the HttpRequestrequest_builder.
  pub fn new(method: impl Into<String>, url: impl AsRef<str>) -> crate::api::Result<Self> {
    Ok(Self {
      method: method.into(),
      url: Url::parse(url.as_ref())?,
      query: None,
      headers: None,
      body: None,
      timeout: None,
      response_type: None,
    })
  }

  /// Sets the request parameters.
  #[must_use]
  pub fn query(mut self, query: HashMap<String, String>) -> Self {
    self.query = Some(query);
    self
  }

  /// Adds a header.
  pub fn header<K, V>(mut self, key: K, value: V) -> crate::api::Result<Self>
  where
    HeaderName: TryFrom<K>,
    <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
    HeaderValue: TryFrom<V>,
    <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
  {
    let key: Result<HeaderName, http::Error> = key.try_into().map_err(Into::into);
    let value: Result<HeaderValue, http::Error> = value.try_into().map_err(Into::into);
    self
      .headers
      .get_or_insert_with(Default::default)
      .0
      .insert(key?, value?);
    Ok(self)
  }

  /// Sets the request headers.
  #[must_use]
  pub fn headers(mut self, headers: header::HeaderMap) -> Self {
    self.headers.replace(HeaderMap(headers));
    self
  }

  /// Sets the request body.
  #[must_use]
  pub fn body(mut self, body: Body) -> Self {
    self.body = Some(body);
    self
  }

  /// Sets the general request timeout.
  #[must_use]
  pub fn timeout(mut self, timeout: Duration) -> Self {
    self.timeout.replace(timeout);
    self
  }

  /// Sets the type of the response. Interferes with the way we read the response.
  #[must_use]
  pub fn response_type(mut self, response_type: ResponseType) -> Self {
    self.response_type = Some(response_type);
    self
  }
}

/// The HTTP response.
#[cfg(feature = "reqwest-client")]
#[derive(Debug)]
pub struct Response(ResponseType, reqwest::Response);
/// The HTTP response.
#[cfg(not(feature = "reqwest-client"))]
#[derive(Debug)]
pub struct Response(ResponseType, attohttpc::Response, Url);

impl Response {
  /// Get the [`StatusCode`] of this Response.
  pub fn status(&self) -> StatusCode {
    self.1.status()
  }

  /// Get the headers of this Response.
  pub fn headers(&self) -> &header::HeaderMap {
    self.1.headers()
  }

  /// Reads the response as raw bytes.
  pub async fn bytes(self) -> crate::api::Result<RawResponse> {
    let status = self.status().as_u16();
    #[cfg(feature = "reqwest-client")]
    let data = self.1.bytes().await?.to_vec();
    #[cfg(not(feature = "reqwest-client"))]
    let data = self.1.bytes()?;
    Ok(RawResponse { status, data })
  }

  #[cfg(not(feature = "reqwest-client"))]
  #[allow(dead_code)]
  pub(crate) fn reader(self) -> attohttpc::ResponseReader {
    let (_, _, reader) = self.1.split();
    reader
  }

  /// Convert the response into a Stream of [`bytes::Bytes`] from the body.
  ///
  /// # Examples
  ///
  /// ```no_run
  /// use futures::StreamExt;
  ///
  /// # async fn run() -> Result<(), Box<dyn std::error::Error>> {
  /// let client = tauri::api::http::ClientBuilder::new().build()?;
  /// let mut stream = client.send(tauri::api::http::HttpRequestBuilder::new("GET", "http://httpbin.org/ip")?)
  ///   .await?
  ///   .bytes_stream();
  ///
  /// while let Some(item) = stream.next().await {
  ///     println!("Chunk: {:?}", item?);
  /// }
  /// # Ok(())
  /// # }
  /// ```
  #[cfg(feature = "reqwest-client")]
  #[allow(dead_code)]
  pub(crate) fn bytes_stream(
    self,
  ) -> impl futures::Stream<Item = crate::api::Result<bytes::Bytes>> {
    use futures::StreamExt;
    self.1.bytes_stream().map(|res| res.map_err(Into::into))
  }

  /// Reads the response.
  ///
  /// Note that the body is serialized to a [`Value`].
  pub async fn read(self) -> crate::api::Result<ResponseData> {
    #[cfg(feature = "reqwest-client")]
    let url = self.1.url().clone();
    #[cfg(not(feature = "reqwest-client"))]
    let url = self.2;

    let mut headers = HashMap::new();
    let mut raw_headers = HashMap::new();
    for (name, value) in self.1.headers() {
      headers.insert(
        name.as_str().to_string(),
        String::from_utf8(value.as_bytes().to_vec())?,
      );
      raw_headers.insert(
        name.as_str().to_string(),
        self
          .1
          .headers()
          .get_all(name)
          .into_iter()
          .map(|v| String::from_utf8(v.as_bytes().to_vec()).map_err(Into::into))
          .collect::<crate::api::Result<Vec<String>>>()?,
      );
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
      raw_headers,
      data,
    })
  }
}

/// A response with raw bytes.
#[non_exhaustive]
#[derive(Debug)]
pub struct RawResponse {
  /// Response status code.
  pub status: u16,
  /// Response bytes.
  pub data: Vec<u8>,
}

/// The response data.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ResponseData {
  /// Response URL. Useful if it followed redirects.
  pub url: Url,
  /// Response status code.
  pub status: u16,
  /// Response headers.
  pub headers: HashMap<String, String>,
  /// Response raw headers.
  pub raw_headers: HashMap<String, Vec<String>>,
  /// Response data.
  pub data: Value,
}

#[cfg(test)]
mod test {
  use super::ClientBuilder;
  use quickcheck::{Arbitrary, Gen};

  impl Arbitrary for ClientBuilder {
    fn arbitrary(g: &mut Gen) -> Self {
      Self {
        max_redirections: Option::arbitrary(g),
        connect_timeout: Option::arbitrary(g),
      }
    }
  }
}
