// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::InvokeResponse;

use crate::api::http::{ClientBuilder, HttpRequestBuilder};
use serde::Deserialize;

#[cfg(http_request)]
use std::{
  collections::HashMap,
  sync::{Arc, Mutex},
};

type ClientId = u32;
#[cfg(http_request)]
type ClientStore = Arc<Mutex<HashMap<ClientId, crate::api::http::Client>>>;

#[cfg(http_request)]
fn clients() -> &'static ClientStore {
  use once_cell::sync::Lazy;
  static STORE: Lazy<ClientStore> = Lazy::new(Default::default);
  &STORE
}

/// The API descriptor.
#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  /// Create a new HTTP client.
  CreateClient { options: Option<ClientBuilder> },
  /// Drop a HTTP client.
  DropClient { client: ClientId },
  /// The HTTP request API.
  HttpRequest {
    client: ClientId,
    options: Box<HttpRequestBuilder>,
  },
}

impl Cmd {
  pub async fn run(self) -> crate::Result<InvokeResponse> {
    match self {
      #[cfg(http_request)]
      Self::CreateClient { options } => {
        let client = options.unwrap_or_default().build()?;
        let mut store = clients().lock().unwrap();
        let id = rand::random::<ClientId>();
        store.insert(id, client);
        Ok(InvokeResponse::from(id))
      }
      #[cfg(not(http_request))]
      Self::CreateClient { .. } => Err(crate::Error::ApiNotAllowlisted(
        "http > request".to_string(),
      )),

      #[cfg(http_request)]
      Self::DropClient { client } => {
        let mut store = clients().lock().unwrap();
        store.remove(&client);
        Ok(().into())
      }
      #[cfg(not(http_request))]
      Self::DropClient { .. } => Err(crate::Error::ApiNotAllowlisted(
        "http > request".to_string(),
      )),

      #[cfg(http_request)]
      Self::HttpRequest { client, options } => {
        return make_request(client, *options).await.map(Into::into);
      }
      #[cfg(not(http_request))]
      Self::HttpRequest { .. } => Err(crate::Error::ApiNotAllowlisted(
        "http > request".to_string(),
      )),
    }
  }
}

/// Makes an HTTP request and resolves the response to the webview
#[cfg(http_request)]
pub async fn make_request(
  client_id: ClientId,
  options: HttpRequestBuilder,
) -> crate::Result<crate::api::http::ResponseData> {
  let client = clients()
    .lock()
    .unwrap()
    .get(&client_id)
    .ok_or(crate::Error::HttpClientNotInitialized)?
    .clone();
  let response = client.send(options).await?;
  Ok(response.read().await?)
}
