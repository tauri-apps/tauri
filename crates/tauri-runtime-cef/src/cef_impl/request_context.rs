// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{
  fs::create_dir_all,
  path::{Component, Path, PathBuf},
  sync::{
    Arc, Mutex,
    atomic::{AtomicBool, Ordering},
  },
  time::Duration,
};

use base64::Engine;
use cef::*;
use sha2::{Digest, Sha256};
use tauri_runtime::webview::WebviewAttributes;
use tauri_utils::Theme;

use crate::cef_impl::request_handler;

#[inline]
fn theme_to_color_variant(theme: Option<Theme>) -> ColorVariant {
  match theme {
    Some(Theme::Dark) => ColorVariant::DARK,
    Some(Theme::Light) => ColorVariant::LIGHT,
    _ => ColorVariant::SYSTEM,
  }
}

pub(crate) fn apply_theme_scheme(request_context: Option<&RequestContext>, theme: Option<Theme>) {
  if let Some(request_context) = request_context {
    request_context.set_chrome_color_scheme(theme_to_color_variant(theme), 0);
  }
}

/// Resolves a CEF-compatible cache path for a per-webview request context.
///
/// CEF requires `RequestContextSettings.cache_path` to be either empty (which
/// puts the context in incognito mode) or an absolute path that is equal to,
/// or a child directory of, `Settings.root_cache_path` (which defaults to
/// `Settings.cache_path` when not set explicitly). Any value outside of that
/// root makes `request_context_create_context` (and downstream browser
/// creation) fail.
///
/// To support an arbitrary [`WebviewAttributes::data_directory`] while
/// honoring this constraint we:
///
/// * use the requested path directly when it already lives under the global
///   cache root, so callers that opt in to a path under the app cache get the
///   exact location they asked for;
/// * join relative paths without parent (`..`) components onto the root cache
///   path (typical short labels); and
/// * otherwise derive a stable direct child folder under `<root>/<hash>` from
///   the requested path, preserving isolation between webviews. Distinct
///   `data_directory` values produce distinct profiles, and the same value
///   maps to the same on-disk profile across runs.
fn resolve_request_context_cache_path(global_cache_path: &Path, data_directory: &Path) -> PathBuf {
  if data_directory.is_absolute() {
    if data_directory.starts_with(global_cache_path) {
      return data_directory.to_path_buf();
    } else {
      log::warn!(
        "data directory is not a child of the global cache path, we will derive a profile hash from it"
      );
    }
  } else if !data_directory
    .components()
    .any(|component| matches!(component, Component::ParentDir))
  {
    return global_cache_path.join(data_directory);
  } else {
    log::warn!(
      "data directory is a relative path with parent components, we will derive a profile hash from it"
    );
  }

  let mut hasher = Sha256::new();
  hasher.update(data_directory.as_os_str().as_encoded_bytes());
  let hash = hasher.finalize();
  let suffix = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(&hash[..16]);
  let path = global_cache_path.join(format!("Profile-{suffix}"));
  log::info!(
    "derived profile hash from data directory: {suffix}, cache path: {}",
    path.display()
  );
  path
}

/// Continuation invoked on the CEF UI thread once the request context's
/// underlying browser context has finished asynchronous initialization.
///
/// Receives a fresh handle to the same [`RequestContext`] that was created in
/// [`request_context_from_webview_attributes`], so the continuation can pass
/// it to `browser_view_create` / `browser_host_create_browser_sync` knowing
/// that `VerifyBrowserContext()` will succeed.
pub(crate) type RequestContextInitContinuation = Box<dyn FnOnce(Option<RequestContext>) + 'static>;

/// Wraps a deferred-init continuation so that it always flips a shared
/// completion flag when it exits, regardless of how it exits (normal return,
/// early `return` on browser-create failure, or panic).
///
/// Returns the completion flag plus the wrapped continuation.
pub(crate) fn deferred_init_continuation<F>(
  work: F,
) -> (Arc<AtomicBool>, RequestContextInitContinuation)
where
  F: FnOnce(Option<RequestContext>) + 'static,
{
  struct Guard(Arc<AtomicBool>);
  impl Drop for Guard {
    fn drop(&mut self) {
      self.0.store(true, Ordering::SeqCst);
    }
  }

  let flag = Arc::new(AtomicBool::new(false));
  let guard = Guard(flag.clone());
  let wrapped: RequestContextInitContinuation = Box::new(move |request_context| {
    let _guard = guard;
    work(request_context);
  });
  (flag, wrapped)
}

/// Block the calling thread until `flag` is `true`.
///
/// Browser creation goes through `RequestContextHandler::on_request_context_initialized`,
/// which CEF always dispatches via `CEF_POST_TASK(CEF_UIT, ...)`. Tauri runs
/// CEF with an external message pump (see `cef::do_message_loop_work` in the
/// runtime's main loop), so the only way for that posted task to actually
/// execute is for someone on the CEF UI thread to keep pumping the loop.
///
/// Two cases:
///
/// 1. We're on the CEF UI thread (typical: app setup, dispatched messages, or
///    inside a CEF callback like `LifeSpanHandler::on_after_created` /
///    `RequestHandler::on_open_url_from_tab`). Pump the message loop ourselves
///    so the `OnRequestContextInitialized` task can run.
///
///    We must enable nestable tasks for the duration of the pump because we
///    may already be running inside another CEF task; without
///    `CefSetNestableTasksAllowed(true)` Chromium's `RunLoop::RunUntilIdle`
///    refuses to dispatch any task to the UI thread, the deferred init never
///    fires, and we'd spin here forever.
///
/// 2. We're on some other thread (e.g. a tokio IPC handler that called the
///    Tauri API directly). The CEF UI thread is running its own pump and will
///    pick up our queued init task on its own; we just block here on a sleep
///    loop until the flag flips. We can't call `do_message_loop_work` from
///    this thread - it asserts on the init thread.
///
/// Spinning here keeps `create_webview` synchronous from the caller's
/// perspective: the function does not return until the browser exists in
/// `state.windows`, so any subsequent dispatcher call (e.g.
/// `webview.open_devtools()`, `webview.on_dev_tools_protocol(...)`) can find
/// the webview.
pub(crate) fn wait_for_deferred_init(flag: &Arc<AtomicBool>) {
  let on_ui_thread = cef::currently_on(cef::sys::cef_thread_id_t::TID_UI.into()) != 0;

  if on_ui_thread {
    let _allow = AllowNestableTasks::enter();
    while !flag.load(Ordering::SeqCst) {
      cef::do_message_loop_work();
    }
  } else {
    while !flag.load(Ordering::SeqCst) {
      std::thread::sleep(Duration::from_millis(1));
    }
  }
}

/// RAII guard that scopes `CefSetNestableTasksAllowed(true)` for the current
/// CEF UI-thread call.
///
/// CEF requires balanced enable/disable calls and explicitly forbids
/// reentrancy at the C++ level (`CHECK(allowed != has_value())`). The guard
/// uses a thread-local depth counter so only the outermost
/// [`wait_for_deferred_init`] on this thread toggles the flag, which makes
/// nesting (e.g. an `on_initialized` continuation that creates another
/// webview) safe.
struct AllowNestableTasks;

impl AllowNestableTasks {
  fn enter() -> Self {
    NESTABLE_TASKS_DEPTH.with(|depth| {
      let current = depth.get();
      if current == 0 {
        cef::set_nestable_tasks_allowed(1);
      }
      depth.set(current + 1);
    });
    Self
  }
}

impl Drop for AllowNestableTasks {
  fn drop(&mut self) {
    NESTABLE_TASKS_DEPTH.with(|depth| {
      let current = depth.get();
      depth.set(current - 1);
      if current == 1 {
        cef::set_nestable_tasks_allowed(0);
      }
    });
  }
}

thread_local! {
  static NESTABLE_TASKS_DEPTH: std::cell::Cell<usize> = const { std::cell::Cell::new(0) };
}

wrap_request_context_handler! {
  struct WebviewRequestContextHandler {
    on_initialized: Arc<Mutex<Option<RequestContextInitContinuation>>>,
  }

  impl RequestContextHandler {
    fn on_request_context_initialized(&self, request_context: Option<&mut RequestContext>) {
      let Some(callback) = self.on_initialized.lock().unwrap().take() else {
        return;
      };
      let request_context = request_context.map(|rc| rc.clone());
      callback(request_context);
    }
  }
}

/// Creates a per-webview [`RequestContext`], registers Tauri's custom URI
/// scheme handler factories on it, and arranges for `on_initialized` to fire
/// once the underlying Chromium `Profile` is fully created.
///
/// CEF only synchronously initializes the request context when its `cache_path`
/// equals `Settings.root_cache_path` (it then reuses the global "Default"
/// profile via `GetPrimaryUserProfile()`) or when the cache_path is empty
/// (off-the-record profile). Any other path (notably the per-`data_directory`
/// case used by Tauri) takes `ChromeBrowserContext::InitializeAsync`'s
/// `CreateProfileAsync` branch which finishes asynchronously. Calling
/// `browser_host_create_browser_sync` synchronously after
/// `request_context_create_context` would then fail
/// `CefRequestContextImpl::VerifyBrowserContext()` and return a null browser.
///
/// Routing browser creation through `on_initialized` keeps a single code path
/// for every cache_path layout: CEF always dispatches the callback through
/// `CEF_POST_TASK(CEF_UIT, ...)`, so even the synchronous-init cases are
/// handled by the same continuation.
///
/// Scheme handler factories are registered here, synchronously after
/// `request_context_create_context` returns, and *before* the
/// `OnRequestContextInitialized` task that drives browser creation is
/// dispatched. `RegisterSchemeHandlerFactory` internally queues its work
/// behind the request context's initialization (`StoreOrTriggerInitCallback`
/// when the browser context is not yet initialized, or an immediate UI -> IO
/// hop otherwise), so by the time the browser finally issues its first
/// navigation against any of these schemes the factories have been wired up
/// on the IO thread.
/// Applies a fixed-server proxy to a request context via the Chromium `proxy`
/// preference. Must be called after the request context has initialized.
fn apply_proxy(request_context: &RequestContext, proxy_url: &url::Url) {
  use cef::{ImplDictionaryValue, ImplValue};

  let scheme = match proxy_url.scheme() {
    "socks5" | "socks5h" => "socks5",
    "socks4" | "socks4a" => "socks4",
    "https" => "https",
    _ => "http",
  };
  let Some(host) = proxy_url.host_str() else {
    log::warn!("ignoring proxy URL without a host: {proxy_url}");
    return;
  };
  let server = match proxy_url.port_or_known_default() {
    Some(port) => format!("{scheme}://{host}:{port}"),
    None => format!("{scheme}://{host}"),
  };

  let pref_name = "proxy";
  if request_context.can_set_preference(Some(&pref_name.into())) != 1 {
    log::warn!("the CEF request context does not allow setting the proxy preference");
    return;
  }

  // Build `{ "mode": "fixed_servers", "server": "<scheme>://<host>:<port>" }`.
  let Some(dict) = cef::dictionary_value_create() else {
    return;
  };
  dict.set_string(Some(&"mode".into()), Some(&"fixed_servers".into()));
  dict.set_string(Some(&"server".into()), Some(&server.as_str().into()));

  let Some(value) = cef::value_create() else {
    return;
  };
  let mut dict = dict;
  value.set_dictionary(Some(&mut dict));

  let mut value = value;
  if request_context.set_preference(Some(&pref_name.into()), Some(&mut value), None) != 1 {
    log::error!("failed to apply the proxy preference to the CEF request context");
  }
}

pub(crate) fn request_context_from_webview_attributes<'a>(
  global_cache_path: &Path,
  webview_attributes: &WebviewAttributes,
  custom_schemes: impl IntoIterator<Item = &'a String>,
  custom_protocol_scheme: &str,
  scheme_registry: request_handler::SchemeRegistry,
  on_initialized: RequestContextInitContinuation,
) -> Option<RequestContext> {
  let cache_path = if webview_attributes.incognito {
    CefStringUtf16::from("")
  } else if let Some(data_directory) = &webview_attributes.data_directory {
    let cache_path = resolve_request_context_cache_path(global_cache_path, data_directory);
    if let Err(error) = create_dir_all(&cache_path) {
      log::error!(
        "failed to create request context cache directory {}: {error}",
        cache_path.display()
      );
    }
    CefStringUtf16::from(cache_path.to_string_lossy().as_ref())
  } else {
    let global_context =
      request_context_get_global_context().expect("Failed to get global request context");
    // global_cache_path does not work here - global_context.cache_path() returns the proper profile path.
    (&global_context.cache_path()).into()
  };

  let settings = RequestContextSettings {
    cache_path,
    ..Default::default()
  };

  // Holds a strong reference to the `RequestContext` until the
  // `on_request_context_initialized` callback fires. CEF keeps the underlying
  // C++ `CefRequestContextImpl` alive during async profile creation through
  // its own bound callbacks, but holding an explicit reference here guarantees
  // we don't race with reference-count releases on shutdown paths.
  let rc_holder: Arc<Mutex<Option<RequestContext>>> = Arc::new(Mutex::new(None));
  let proxy_url = webview_attributes.proxy_url.clone();
  let wrapped_callback: RequestContextInitContinuation = Box::new({
    let rc_holder = rc_holder.clone();
    move |rc| {
      // The proxy preference can only be set once the request context's
      // underlying profile has finished initializing, which is exactly what
      // this continuation signals.
      if let (Some(rc), Some(proxy_url)) = (rc.as_ref(), proxy_url.as_ref()) {
        apply_proxy(rc, proxy_url);
      }
      on_initialized(rc);
      let _released = rc_holder.lock().unwrap().take();
    }
  });

  let mut handler = WebviewRequestContextHandler::new(Arc::new(Mutex::new(Some(wrapped_callback))));
  let request_context = request_context_create_context(Some(&settings), Some(&mut handler));
  *rc_holder.lock().unwrap() = request_context.clone();

  if let Some(request_context) = request_context.as_ref() {
    for scheme in custom_schemes {
      request_context.register_scheme_handler_factory(
        Some(&custom_protocol_scheme.into()),
        Some(&format!("{scheme}.localhost").as_str().into()),
        Some(&mut request_handler::UriSchemeHandlerFactory::new(
          scheme_registry.clone(),
          scheme.clone(),
        )),
      );
    }
  }

  request_context
}
