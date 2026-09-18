// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  collections::HashMap,
  str::FromStr,
  sync::{
    Arc, Mutex, OnceLock,
    atomic::{AtomicU32, AtomicUsize, Ordering},
  },
};

use http::HeaderMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use url::Url;

use crate::{
  Manager, Runtime, State, Webview, command,
  ipc::{CommandArg, CommandItem, InvokeMessage},
  plugin::{Builder as PluginBuilder, TauriPlugin},
};

use super::{
  CallbackFn, InvokeError, InvokeResponseBody, IpcResponse, Request, Response,
  format_callback::format_raw_js,
};

pub const IPC_PAYLOAD_PREFIX: &str = "__CHANNEL__:";
pub const CHANNEL_PLUGIN_NAME: &str = "channel";
pub const FETCH_CHANNEL_DATA_COMMAND: &str = "plugin:channel|fetch";
const CHANNEL_ID_HEADER_NAME: &str = "Tauri-Channel-Id";

/// Maximum size a JSON we should send directly without going through the fetch process
// 8192 byte JSON payload runs roughly 2x faster through eval than through fetch on WebView2 v135
const MAX_JSON_DIRECT_EXECUTE_THRESHOLD: usize = 8192;
// 1024 byte payload runs  roughly 30% faster through eval than through fetch on macOS
const MAX_RAW_DIRECT_EXECUTE_THRESHOLD: usize = 1024;

static CHANNEL_COUNTER: AtomicU32 = AtomicU32::new(0);
static CHANNEL_DATA_COUNTER: AtomicU32 = AtomicU32::new(0);

/// Maps a channel data id to the pending payload and the label of the webview that must fetch it.
#[derive(Default, Clone)]
pub struct ChannelDataIpcQueue(Arc<Mutex<HashMap<u32, (String, InvokeResponseBody)>>>);

impl ChannelDataIpcQueue {
  /// Queues `body` for `webview_label` and returns the id the webview must fetch it with.
  fn push(&self, webview_label: &str, body: InvokeResponseBody) -> u32 {
    let data_id = CHANNEL_DATA_COUNTER.fetch_add(1, Ordering::Relaxed);
    self
      .0
      .lock()
      .unwrap()
      .insert(data_id, (webview_label.to_string(), body));
    data_id
  }

  /// Takes the payload queued under `data_id` for `webview_label`.
  ///
  /// Entries queued for another webview are left in place.
  fn take(&self, data_id: u32, webview_label: &str) -> Option<InvokeResponseBody> {
    let mut queue = self.0.lock().unwrap();
    match queue.get(&data_id) {
      Some((label, _)) if label == webview_label => queue.remove(&data_id).map(|(_, body)| body),
      _ => None,
    }
  }

  /// Drops every payload queued for `webview_label`.
  pub(crate) fn purge_webview(&self, webview_label: &str) {
    self
      .0
      .lock()
      .unwrap()
      .retain(|_, (label, _)| label != webview_label);
  }
}

/// Whether the payload is large enough for the fetch transport to beat an inline eval.
fn exceeds_inline_threshold(body: &InvokeResponseBody) -> bool {
  match body {
    InvokeResponseBody::Json(json) => json.len() >= MAX_JSON_DIRECT_EXECUTE_THRESHOLD,
    InvokeResponseBody::Raw(bytes) => bytes.len() >= MAX_RAW_DIRECT_EXECUTE_THRESHOLD,
  }
}

/// JavaScript expression evaluating to the payload, for the inline transport.
fn inline_payload(body: InvokeResponseBody) -> crate::Result<String> {
  Ok(match body {
    InvokeResponseBody::Json(json) => json,
    InvokeResponseBody::Raw(bytes) => {
      format!("new Uint8Array({}).buffer", serde_json::to_string(&bytes)?)
    }
  })
}

/// Queues `body` for the webview and returns the JavaScript that fetches it and runs the callback
/// with `result`, an expression over `response` (the fetched payload).
fn fetch_payload<R: Runtime>(
  webview: &Webview<R>,
  callback_id: u32,
  body: InvokeResponseBody,
  result: &str,
) -> String {
  let data_id = webview
    .state::<ChannelDataIpcQueue>()
    .push(webview.label(), body);
  format!(
    "window.__TAURI_INTERNALS__.invoke('{FETCH_CHANNEL_DATA_COMMAND}', null, {{ headers: {{ '{CHANNEL_ID_HEADER_NAME}': '{data_id}' }} }}).then((response) => window.__TAURI_INTERNALS__.runCallback({callback_id}, {result})).catch(console.error)",
  )
}

/// Whether the webview is allowed to fetch queued channel data, i.e. whether its capability
/// grants `core:channel:allow-fetch` on the page at `url` (the current page when `None`).
///
/// A webview that cannot fetch gets its large payloads inline instead: queueing them would leak
/// the entry and, since the JavaScript side delivers strictly by index, park every later message
/// of the channel forever.
fn fetch_allowed<R: Runtime>(webview: &Webview<R>, url: Option<&Url>) -> bool {
  let access = match url {
    Some(url) => webview.resolve_command_access_on(FETCH_CHANNEL_DATA_COMMAND, url),
    // an unknown URL is treated as denied
    None => webview
      .resolve_command_access(FETCH_CHANNEL_DATA_COMMAND)
      .ok()
      .flatten(),
  };

  if access.is_none() {
    static WARNED: std::sync::Once = std::sync::Once::new();
    WARNED.call_once(|| {
      let label = webview.label();
      #[cfg(feature = "tracing")]
      tracing::warn!(
        "channel payload for webview `{label}` sent inline because `core:channel:allow-fetch` is not granted to it; add `core:channel:default` (or `core:default`) to its capability for faster large payloads"
      );
      #[cfg(not(feature = "tracing"))]
      eprintln!(
        "channel payload for webview `{label}` sent inline because `core:channel:allow-fetch` is not granted to it; add `core:channel:default` (or `core:default`) to its capability for faster large payloads"
      );
    });
  }

  access.is_some()
}

/// An IPC channel.
pub struct Channel<TSend = InvokeResponseBody> {
  inner: Arc<ChannelInner>,
  phantom: std::marker::PhantomData<TSend>,
}

#[cfg(feature = "specta")]
const _: () = {
  #[derive(specta::Type)]
  #[specta(remote = super::Channel)]
  #[allow(dead_code, non_camel_case_types)]
  struct TAURI_CHANNEL<TSend>(std::marker::PhantomData<TSend>);
};

impl<TSend> Clone for Channel<TSend> {
  fn clone(&self) -> Self {
    Self {
      inner: self.inner.clone(),
      phantom: self.phantom,
    }
  }
}

type OnDropFn = Option<Box<dyn Fn() + Send + Sync + 'static>>;
type OnMessageFn = Box<dyn Fn(InvokeResponseBody) -> crate::Result<()> + Send + Sync>;

struct ChannelInner {
  id: u32,
  on_message: OnMessageFn,
  on_drop: OnDropFn,
}

impl Drop for ChannelInner {
  fn drop(&mut self) {
    if let Some(on_drop) = &self.on_drop {
      on_drop();
    }
  }
}

impl<TSend> Serialize for Channel<TSend> {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(&format!("{IPC_PAYLOAD_PREFIX}{}", self.inner.id))
  }
}

/// The ID of a channel that was defined on the JavaScript layer.
///
/// Useful when expecting [`Channel`] as part of a JSON object instead of a top-level command argument.
///
/// # Examples
///
/// ```rust
/// use tauri::{ipc::JavaScriptChannelId, Runtime, Webview};
///
/// #[derive(serde::Deserialize)]
/// #[serde(rename_all = "camelCase")]
/// struct Button {
///   label: String,
///   on_click: JavaScriptChannelId,
/// }
///
/// #[tauri::command]
/// fn add_button<R: Runtime>(webview: Webview<R>, button: Button) {
///   let channel = button.on_click.channel_on(webview);
///   channel.send("clicked").unwrap();
/// }
/// ```
pub struct JavaScriptChannelId(CallbackFn);

impl FromStr for JavaScriptChannelId {
  type Err = &'static str;

  fn from_str(s: &str) -> Result<Self, Self::Err> {
    s.strip_prefix(IPC_PAYLOAD_PREFIX)
      .ok_or("invalid channel string")
      .and_then(|id| id.parse().map_err(|_| "invalid channel ID"))
      .map(|id| Self(CallbackFn(id)))
  }
}

impl JavaScriptChannelId {
  /// Gets a [`Channel`] for this channel ID on the given [`Webview`].
  pub fn channel_on<R: Runtime, TSend>(&self, webview: Webview<R>) -> Channel<TSend> {
    self.channel_on_page(webview, None)
  }

  /// Gets a [`Channel`] for this channel ID on the given [`Webview`], whose page is at `url`
  /// when known (saves asking the runtime for it when the first large payload is sent).
  fn channel_on_page<R: Runtime, TSend>(
    &self,
    webview: Webview<R>,
    url: Option<Url>,
  ) -> Channel<TSend> {
    let callback_fn = self.0;
    let callback_id = callback_fn.0;

    let counter = Arc::new(AtomicUsize::new(0));
    let counter_clone = counter.clone();
    let webview_clone = webview.clone();
    // resolved on the first large payload and kept for the channel lifetime: the channel is bound
    // to the page that created it (its JavaScript callback does not survive a navigation)
    let fetch_allowed = OnceLock::new();

    Channel::new_with_id(
      callback_id,
      Box::new(move |body| {
        let current_index = counter.fetch_add(1, Ordering::Relaxed);

        if let Some(interceptor) = &webview.manager.channel_interceptor
          && interceptor(&webview, callback_fn, current_index, &body)
        {
          return Ok(());
        }

        // use the fetch API to speed up larger payloads
        let js = if exceeds_inline_threshold(&body)
          && *fetch_allowed.get_or_init(|| self::fetch_allowed(&webview, url.as_ref()))
        {
          fetch_payload(
            &webview,
            callback_id,
            body,
            &format!("{{ message: response, index: {current_index} }}"),
          )
        } else {
          let payload = inline_payload(body)?;
          format_raw_js(
            callback_id,
            format!("{{ message: {payload}, index: {current_index} }}"),
          )
        };
        webview.eval(js)?;

        Ok(())
      }),
      Some(Box::new(move || {
        let current_index = counter_clone.load(Ordering::Relaxed);
        let _ = webview_clone.eval(format_raw_js(
          callback_id,
          format!("{{ end: true, index: {current_index} }}"),
        ));
      })),
    )
  }
}

impl<'de> Deserialize<'de> for JavaScriptChannelId {
  fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    let value: String = Deserialize::deserialize(deserializer)?;
    Self::from_str(&value).map_err(|_| {
      serde::de::Error::custom(format!(
        "invalid channel value `{value}`, expected a string in the `{IPC_PAYLOAD_PREFIX}ID` format"
      ))
    })
  }
}

impl<TSend> Channel<TSend> {
  /// Creates a new channel with the given message handler.
  pub fn new<F: Fn(InvokeResponseBody) -> crate::Result<()> + Send + Sync + 'static>(
    on_message: F,
  ) -> Self {
    Self::new_with_id(
      CHANNEL_COUNTER.fetch_add(1, Ordering::Relaxed),
      Box::new(on_message),
      None,
    )
  }

  fn new_with_id(id: u32, on_message: OnMessageFn, on_drop: OnDropFn) -> Self {
    #[allow(clippy::let_and_return)]
    let channel = Self {
      inner: Arc::new(ChannelInner {
        id,
        on_message,
        on_drop,
      }),
      phantom: Default::default(),
    };

    #[cfg(mobile)]
    crate::plugin::mobile::register_channel(Channel {
      inner: channel.inner.clone(),
      phantom: Default::default(),
    });

    channel
  }

  /// Channel that responds to the invoke request `callback` was sent from, by the page at `url`.
  ///
  /// This is used from the IPC handler.
  pub(crate) fn from_callback_fn<R: Runtime>(
    webview: Webview<R>,
    callback: CallbackFn,
    url: &Url,
  ) -> Self {
    let callback_id = callback.0;
    let url = url.clone();
    Channel::new_with_id(
      callback_id,
      Box::new(move |body| {
        // use the fetch API to speed up larger response payloads
        let js = if exceeds_inline_threshold(&body) && fetch_allowed(&webview, Some(&url)) {
          fetch_payload(&webview, callback_id, body, "response")
        } else {
          format_raw_js(callback_id, inline_payload(body)?)
        };
        webview.eval(js)?;

        Ok(())
      }),
      None,
    )
  }

  /// The channel identifier.
  pub fn id(&self) -> u32 {
    self.inner.id
  }

  /// Sends the given data through the channel.
  pub fn send(&self, data: TSend) -> crate::Result<()>
  where
    TSend: IpcResponse,
  {
    (self.inner.on_message)(data.body()?)
  }
}

impl<'de, R: Runtime, TSend> CommandArg<'de, R> for Channel<TSend> {
  /// Grabs the [`Webview`] from the [`CommandItem`] and returns the associated [`Channel`].
  fn from_command(command: CommandItem<'de, R>) -> Result<Self, InvokeError> {
    let name = command.name;
    let arg = command.key;
    let webview = command.message.webview();
    let url = command.message.url.clone();
    let value: String =
      Deserialize::deserialize(command).map_err(|e| crate::Error::InvalidArgs(name, arg, e))?;
    JavaScriptChannelId::from_str(&value)
      .map(|id| id.channel_on_page(webview, Some(url)))
      .map_err(|_| {
        InvokeError::from(format!(
	        "invalid channel value `{value}`, expected a string in the `{IPC_PAYLOAD_PREFIX}ID` format"
	      ))
      })
  }
}

fn channel_data_id(headers: &HeaderMap) -> Option<u32> {
  headers
    .get(CHANNEL_ID_HEADER_NAME)?
    .to_str()
    .ok()?
    .parse()
    .ok()
}

/// Drops the payload a rejected `plugin:channel|fetch` request asked for, if it was queued for
/// the requesting webview.
pub(crate) fn discard_rejected_fetch<R: Runtime>(message: &InvokeMessage<R>) {
  if let Some(data_id) = channel_data_id(&message.headers)
    && let Some(queue) = message.webview.try_state::<ChannelDataIpcQueue>()
  {
    queue.take(data_id, message.webview.label());
  }
}

#[command(root = "crate")]
fn fetch<R: Runtime>(
  webview: Webview<R>,
  request: Request<'_>,
  cache: State<'_, ChannelDataIpcQueue>,
) -> Result<Response, &'static str> {
  let data_id = channel_data_id(request.headers()).ok_or("missing channel id header")?;
  cache
    .take(data_id, webview.label())
    .map(Response::new)
    .ok_or("data not found")
}

pub fn plugin<R: Runtime>() -> TauriPlugin<R> {
  PluginBuilder::new(CHANNEL_PLUGIN_NAME)
    .invoke_handler(crate::generate_handler![
      #![plugin(channel)]
      fetch
    ])
    .build()
}

#[cfg(test)]
mod tests {
  use std::str::FromStr;

  use super::{
    CHANNEL_ID_HEADER_NAME, ChannelDataIpcQueue, FETCH_CHANNEL_DATA_COMMAND, IPC_PAYLOAD_PREFIX,
    JavaScriptChannelId, MAX_JSON_DIRECT_EXECUTE_THRESHOLD,
  };
  use crate::{
    Manager, WebviewWindow,
    ipc::{CallbackFn, Channel, InvokeBody, InvokeResponseBody, RuntimeAuthority},
    sealed::ManagerBase,
    test::{INVOKE_KEY, MockRuntime, get_ipc_response, mock_builder, mock_context, noop_assets},
    webview::InvokeRequest,
  };
  use tauri_utils::acl::resolved::{Resolved, ResolvedCommand};

  fn fetch_request(data_id: u32) -> InvokeRequest {
    let mut headers = http::HeaderMap::new();
    headers.insert(CHANNEL_ID_HEADER_NAME, data_id.to_string().parse().unwrap());
    InvokeRequest {
      cmd: FETCH_CHANNEL_DATA_COMMAND.into(),
      callback: CallbackFn(0),
      error: CallbackFn(1),
      url: "tauri://localhost".parse().unwrap(),
      body: InvokeBody::default(),
      headers,
      invoke_key: INVOKE_KEY.to_string(),
    }
  }

  fn queue_data(app: &crate::App<MockRuntime>, data_id: u32, webview_label: &str, payload: &[u8]) {
    app.state::<ChannelDataIpcQueue>().0.lock().unwrap().insert(
      data_id,
      (
        webview_label.to_string(),
        InvokeResponseBody::Raw(payload.to_vec()),
      ),
    );
  }

  fn queued_labels(app: &crate::App<MockRuntime>) -> Vec<String> {
    let queue = app.state::<ChannelDataIpcQueue>();
    let queue = queue.0.lock().unwrap();
    let mut labels = queue
      .values()
      .map(|(label, _)| label.clone())
      .collect::<Vec<_>>();
    labels.sort();
    labels
  }

  /// A context whose ACL grants `plugin:channel|fetch` to every webview.
  fn context_allowing_fetch() -> crate::Context<MockRuntime> {
    let mut context = mock_context(noop_assets());
    *context.runtime_authority_mut() = RuntimeAuthority::new(
      Default::default(),
      Resolved {
        allowed_commands: [(
          FETCH_CHANNEL_DATA_COMMAND.to_string(),
          vec![ResolvedCommand {
            windows: vec![glob::Pattern::new("*").unwrap()],
            webviews: vec![glob::Pattern::new("*").unwrap()],
            ..Default::default()
          }],
        )]
        .into_iter()
        .collect(),
        ..Default::default()
      },
    );
    context
  }

  fn build_webview(app: &crate::App<MockRuntime>, label: &str) -> WebviewWindow<MockRuntime> {
    crate::WebviewWindowBuilder::new(app, label, Default::default())
      .build()
      .unwrap()
  }

  fn js_channel(webview: &WebviewWindow<MockRuntime>, callback_id: u32) -> Channel {
    JavaScriptChannelId::from_str(&format!("{IPC_PAYLOAD_PREFIX}{callback_id}"))
      .unwrap()
      .channel_on(webview.as_ref().clone())
  }

  fn last_evaluated_script(webview: &WebviewWindow<MockRuntime>) -> String {
    webview
      .webview
      .webview
      .dispatcher
      .last_evaluated_script()
      .expect("no script was evaluated")
  }

  fn large_json() -> String {
    format!("\"{}\"", "x".repeat(MAX_JSON_DIRECT_EXECUTE_THRESHOLD))
  }

  #[test]
  fn fetch_is_gated_by_the_acl() {
    let app = mock_builder().build(mock_context(noop_assets())).unwrap();
    let webview = build_webview(&app, "main");

    queue_data(&app, 1, "other", b"payload");

    let err = get_ipc_response(&webview, fetch_request(1)).unwrap_err();
    assert!(
      err.to_string().contains("not allowed"),
      "fetch must be rejected when core:channel is not in the ACL, got: {err}"
    );
    // another webview's data is left untouched by a rejected request
    assert_eq!(queued_labels(&app), ["other"]);
  }

  #[test]
  fn large_payloads_are_sent_inline_when_fetch_is_not_allowed() {
    let app = mock_builder().build(mock_context(noop_assets())).unwrap();
    let webview = build_webview(&app, "main");
    let channel = js_channel(&webview, 7);

    let payload = large_json();
    channel
      .send(InvokeResponseBody::Json(payload.clone()))
      .unwrap();

    // nothing is queued for a webview that could never fetch it
    assert!(queued_labels(&app).is_empty());
    let script = last_evaluated_script(&webview);
    assert!(
      !script.contains(FETCH_CHANNEL_DATA_COMMAND),
      "payload must not go through the fetch command: {script}"
    );
    assert!(
      script.contains(&format!("{{ message: {payload}, index: 0 }}")),
      "payload must be delivered inline with its index: {script}"
    );

    // a later message keeps the ordering
    channel
      .send(InvokeResponseBody::Json("\"small\"".into()))
      .unwrap();
    assert!(last_evaluated_script(&webview).contains("{ message: \"small\", index: 1 }"));
  }

  #[test]
  fn large_payloads_are_queued_when_fetch_is_allowed() {
    let app = mock_builder().build(context_allowing_fetch()).unwrap();
    let webview = build_webview(&app, "main");
    let channel = js_channel(&webview, 7);

    let payload = large_json();
    channel
      .send(InvokeResponseBody::Json(payload.clone()))
      .unwrap();

    // the payload is queued for this webview and the webview is told to fetch it
    assert_eq!(queued_labels(&app), ["main"]);
    let script = last_evaluated_script(&webview);
    assert!(
      script.contains(FETCH_CHANNEL_DATA_COMMAND) && !script.contains(&payload),
      "payload must go through the fetch command: {script}"
    );

    let data_id = app
      .state::<ChannelDataIpcQueue>()
      .0
      .lock()
      .unwrap()
      .keys()
      .next()
      .copied()
      .unwrap();
    let response = get_ipc_response(&webview, fetch_request(data_id)).unwrap();
    assert_eq!(response, InvokeResponseBody::Json(payload));
    assert!(queued_labels(&app).is_empty());
  }

  #[test]
  fn fetch_serves_the_queued_data_when_allowed() {
    let app = mock_builder().build(context_allowing_fetch()).unwrap();
    let webview = build_webview(&app, "main");

    queue_data(&app, 2, "main", b"payload");

    let response = get_ipc_response(&webview, fetch_request(2)).unwrap();
    assert_eq!(response, InvokeResponseBody::Raw(b"payload".to_vec()));
    // the data is only served once
    let err = get_ipc_response(&webview, fetch_request(2)).unwrap_err();
    assert_eq!(err, serde_json::Value::String("data not found".into()));
  }

  #[test]
  fn fetch_rejects_another_webviews_data() {
    let app = mock_builder().build(context_allowing_fetch()).unwrap();
    let main = build_webview(&app, "main");
    let other = build_webview(&app, "other");

    queue_data(&app, 3, "main", b"payload");

    let err = get_ipc_response(&other, fetch_request(3)).unwrap_err();
    assert_eq!(err, serde_json::Value::String("data not found".into()));
    // the entry is still there for the webview it was queued for
    assert_eq!(queued_labels(&app), ["main"]);
    let response = get_ipc_response(&main, fetch_request(3)).unwrap();
    assert_eq!(response, InvokeResponseBody::Raw(b"payload".to_vec()));
  }

  #[test]
  fn rejected_fetch_discards_the_webviews_queued_data() {
    let app = mock_builder().build(mock_context(noop_assets())).unwrap();
    let webview = build_webview(&app, "main");

    // queued while the fetch was still allowed, then the webview lost the permission (navigation)
    queue_data(&app, 4, "main", b"payload");

    let err = get_ipc_response(&webview, fetch_request(4)).unwrap_err();
    assert!(err.to_string().contains("not allowed"), "got: {err}");
    assert!(queued_labels(&app).is_empty());
  }

  #[test]
  fn closing_a_webview_purges_its_queued_data() {
    let app = mock_builder().build(context_allowing_fetch()).unwrap();
    let _main = build_webview(&app, "main");
    let _other = build_webview(&app, "other");

    queue_data(&app, 5, "main", b"payload");
    queue_data(&app, 6, "other", b"payload");

    app.manager().on_window_close("main");
    assert_eq!(queued_labels(&app), ["other"]);
  }
}
