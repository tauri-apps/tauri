use crate::{async_runtime::Mutex, ApplicationDispatcherExt};

use once_cell::sync::Lazy;
use serde::Deserialize;
use tauri_api::http::{Client, ClientBuilder, HttpRequestBuilder};

use std::{collections::HashMap, sync::Arc};

type ClientId = u32;
type ClientStore = Arc<Mutex<HashMap<ClientId, Client>>>;

fn clients() -> &'static ClientStore {
  static STORE: Lazy<ClientStore> = Lazy::new(Default::default);
  &STORE
}

/// The API descriptor.
#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  /// Create a new HTTP client.
  CreateClient {
    options: Option<ClientBuilder>,
    callback: String,
    error: String,
  },
  /// Drop a HTTP client.
  DropClient { client: ClientId },
  /// The HTTP request API.
  HttpRequest {
    client: ClientId,
    options: Box<HttpRequestBuilder>,
    callback: String,
    error: String,
  },
}

impl Cmd {
  pub async fn run<D: ApplicationDispatcherExt + 'static>(
    self,
    webview_manager: &crate::WebviewManager<D>,
  ) {
    match self {
      Self::CreateClient {
        options,
        callback,
        error,
      } => {
        crate::execute_promise(
          webview_manager,
          async move {
            let client = options.unwrap_or_default().build()?;
            let mut store = clients().lock().await;
            let id = rand::random::<ClientId>();
            store.insert(id, client);
            Ok(id)
          },
          callback,
          error,
        )
        .await;
      }
      Self::DropClient { client } => {
        let mut store = clients().lock().await;
        store.remove(&client);
      }
      Self::HttpRequest {
        client,
        options,
        callback,
        error,
      } => {
        #[cfg(http_request)]
        make_request(webview_manager, client, *options, callback, error).await;
        #[cfg(not(http_request))]
        allowlist_error(webview_manager, error, "httpRequest");
      }
    }
  }
}

/// Makes an HTTP request and resolves the response to the webview
pub async fn make_request<D: ApplicationDispatcherExt>(
  webview_manager: &crate::WebviewManager<D>,
  client_id: ClientId,
  options: HttpRequestBuilder,
  callback: String,
  error: String,
) {
  crate::execute_promise(
    webview_manager,
    async move {
      let client = clients()
        .lock()
        .await
        .get(&client_id)
        .ok_or(crate::Error::HttpClientNotInitialized)?
        .clone();
      let response = options.send(&client).await?;
      Ok(response.read().await?)
    },
    callback,
    error,
  )
  .await;
}
