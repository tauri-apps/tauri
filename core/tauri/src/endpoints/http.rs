// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::InvokeContext;
use crate::Runtime;
use serde::Deserialize;
use tauri_macros::{module_command_handler, CommandModule};

#[cfg(http_request)]
use std::{
  collections::HashMap,
  sync::{Arc, Mutex},
};

#[cfg(http_request)]
use crate::api::http::{ClientBuilder, HttpRequestBuilder, ResponseData};
#[cfg(not(http_request))]
type ClientBuilder = ();
#[cfg(not(http_request))]
type HttpRequestBuilder = ();
#[cfg(not(http_request))]
type ResponseData = ();

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
#[derive(Deserialize, CommandModule)]
#[cmd(async)]
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
  #[module_command_handler(http_request, "http > request")]
  async fn create_client<R: Runtime>(
    _context: InvokeContext<R>,
    options: Option<ClientBuilder>,
  ) -> crate::Result<ClientId> {
    let client = options.unwrap_or_default().build()?;
    let mut store = clients().lock().unwrap();
    let id = rand::random::<ClientId>();
    store.insert(id, client);
    Ok(id)
  }

  #[module_command_handler(http_request, "http > request")]
  async fn drop_client<R: Runtime>(
    _context: InvokeContext<R>,
    client: ClientId,
  ) -> crate::Result<()> {
    let mut store = clients().lock().unwrap();
    store.remove(&client);
    Ok(())
  }

  #[module_command_handler(http_request, "http > request")]
  async fn http_request<R: Runtime>(
    context: InvokeContext<R>,
    client_id: ClientId,
    options: Box<HttpRequestBuilder>,
  ) -> crate::Result<ResponseData> {
    use crate::Manager;
    if context
      .window
      .state::<crate::Scopes>()
      .http
      .is_allowed(&options.url)
    {
      let client = clients()
        .lock()
        .unwrap()
        .get(&client_id)
        .ok_or(crate::Error::HttpClientNotInitialized)?
        .clone();
      let response = client.send(*options).await?;
      Ok(response.read().await?)
    } else {
      Err(crate::Error::UrlNotAllowed(options.url))
    }
  }
}

#[cfg(test)]
mod tests {
  use super::{ClientBuilder, ClientId};

  #[tauri_macros::module_command_test(http_request, "http > request", async)]
  #[quickcheck_macros::quickcheck]
  fn create_client(options: Option<ClientBuilder>) {
    assert!(crate::async_runtime::block_on(super::Cmd::create_client(
      crate::test::mock_invoke_context(),
      options
    ))
    .is_ok());
  }

  #[tauri_macros::module_command_test(http_request, "http > request", async)]
  #[quickcheck_macros::quickcheck]
  fn drop_client(client_id: ClientId) {
    crate::async_runtime::block_on(async move {
      assert!(
        super::Cmd::drop_client(crate::test::mock_invoke_context(), client_id)
          .await
          .is_ok()
      );
      let id = super::Cmd::create_client(crate::test::mock_invoke_context(), None)
        .await
        .unwrap();
      assert!(
        super::Cmd::drop_client(crate::test::mock_invoke_context(), id)
          .await
          .is_ok()
      );
    });
  }
}
