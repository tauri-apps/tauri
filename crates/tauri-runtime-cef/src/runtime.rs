// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

#![allow(clippy::arc_with_non_send_sync)]
#![allow(clippy::too_many_arguments)]

use std::{
  collections::HashMap,
  fmt,
  fs::create_dir_all,
  path::PathBuf,
  sync::{
    Arc, Mutex,
    atomic::{AtomicBool, AtomicPtr, AtomicU32, Ordering},
    mpsc::{self, Receiver, Sender},
  },
  time::Duration,
};

use cef::*;
use raw_window_handle::{DisplayHandle, HasDisplayHandle};
use tauri_runtime::{
  DeviceEventFilter, Error, EventLoopProxy, ExitRequestedEventAction, Result, RunEvent, Runtime,
  RuntimeHandle, RuntimeInitArgs, UserEvent,
  dpi::PhysicalPosition,
  monitor::Monitor,
  webview::{DetachedWebview, PendingWebview},
  window::{
    DetachedWindow, DragDropEvent, PendingWindow, RawWindow, WebviewEvent, WindowEvent, WindowId,
  },
};
use tauri_utils::Theme;
use winit::{
  application::ApplicationHandler,
  event::{StartCause, WindowEvent as WinitWindowEvent},
  event_loop::{
    ActiveEventLoop, EventLoop, EventLoopBuilder, EventLoopProxy as WinitEventLoopProxy,
  },
  window::WindowId as WinitWindowId,
};

use crate::external_message_pump::CefExternalPump;
use crate::platform::EventLoopExt;
use crate::{
  cef_impl::{client as browser_client, ipc, request_handler},
  macros::wrap_with_args,
  webview::{
    self, AppWebview, CefWebviewAttributes, CefWebviewDispatcher, Webview, WebviewMessage,
    create_webview_detached,
  },
  window::{
    AppWindow, CefWindowDispatcher, WindowMessage, create_window_detached,
    winit_monitor_to_tauri_monitor, winit_theme_to_tauri_theme,
  },
  window_handle::SendRawDisplayHandle,
};
#[cfg(target_os = "macos")]
use winit::platform::macos::EventLoopBuilderExtMacOS;
#[cfg(windows)]
use winit::platform::windows::EventLoopBuilderExtWindows;
#[cfg(any(
  target_os = "linux",
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "netbsd",
  target_os = "openbsd"
))]
use winit::platform::x11::EventLoopBuilderExtX11;

/// Customizes the CEF settings before initialization, see [`Cef::with_settings`].
type SettingsCallback = dyn FnOnce(&mut cef::Settings) + Send + Sync;

/// The `cef` crate used by this runtime, re-exported for convenience.
///
/// # Stability
///
/// The cef crate follows the Chromium Embedded Framework interface and there is
/// no API stability guarantees. The crate will be updated frequently, usually
/// in minor releases when a known breaking change is discovered.
pub use cef;

/// Which key Chromium uses to encrypt the little it stores encrypted.
///
/// Chromium's `os_crypt` layer encrypts **cookies and saved passwords** only. Every
/// other piece of web storage — `localStorage`, IndexedDB, Cache Storage, service worker
/// registrations — is written to the cache directory unencrypted whichever variant you
/// pick here, exactly as it is under wry's WebKitGTK and WebView2 backends. So this
/// setting decides how a cookie jar is protected at rest, and nothing else.
///
/// The default, [`SecretStorage::Auto`], skips the OS secret store in development builds
/// (`tauri::is_dev()`), where it is a recurring annoyance, and keeps it in release
/// builds, where it is the only thing protecting the cookie jar. What it skips differs
/// per platform:
///
/// - on macOS, `os_crypt` stores a random key in a shared "Chromium Safe Storage"
///   keychain item whose ACL is bound to the code signature of the process that reads
///   it. Ad-hoc-signed development builds get a new signature on every rebuild, so macOS
///   puts up the keychain password prompt again after every `cargo build`.
/// - on Linux, `os_crypt` asks the D-Bus secret portal, libsecret or KWallet for the
///   key, which pops a keyring-unlock dialog the first time an app runs.
///
/// A release build that has to run where there is no secret store at all — a headless
/// session, a container, a CI image — needs [`SecretStorage::Mock`], because there `Auto`
/// asks for a store that is not there. That is the deliberate trade: Chromium's own
/// default and Electron's are to use the system store, and shipping `basic` by default
/// would leave every packaged Linux application's cookies readable by anything that can
/// read the cache directory.
///
/// # Security
///
/// The mock keychain (`--use-mock-keychain`) and the Linux `basic` password store do not
/// derive a secret key: they encrypt with a key derived from a **hard-coded constant**
/// compiled into Chromium (`mock_password` and `peanuts` respectively). Both constants
/// are public, so cookies encrypted with them have **no meaningful protection at rest** —
/// anyone who can read the cache directory can decrypt them. Choosing one of them for a
/// release build gives up the same protection wry already lacks on Linux, where
/// WebKitGTK stores cookies in plain text; it is a floor to fall back to, not a default.
///
/// # Switching modes invalidates stored cookies
///
/// Cookies encrypted with one key cannot be read back with another, and development and
/// release builds share the same default cache directory
/// (`{user cache}/{identifier}/cef`). Moving an app between [`SecretStorage::Mock`] and
/// [`SecretStorage::System`] — including the implicit move [`SecretStorage::Auto`] makes
/// when a dev build is followed by a release build — therefore drops the cookies stored
/// under the previous key, logging users out. Set [`Cef::root_cache_path`] to separate
/// the two if that matters.
///
/// An application that ships [`SecretStorage::Mock`] to survive keyring-less systems and
/// later moves to [`SecretStorage::System`] hits the same thing in the other direction:
/// a jar encrypted with `peanuts` cannot be read back under a libsecret or KWallet key.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SecretStorage {
  /// Skip the OS secret store in development builds (`tauri::is_dev()`) and use it in
  /// release builds: `--use-mock-keychain` on macOS and `--password-store=basic` on
  /// Linux, in development only. Windows is untouched and keeps using DPAPI.
  #[default]
  Auto,
  /// Always encrypt with Chromium's hard-coded constant: `--use-mock-keychain` on macOS,
  /// `--password-store=basic` on Linux. Windows is untouched and keeps using DPAPI.
  ///
  /// Read the security note on [`SecretStorage`] before shipping this in a release
  /// build: the key is a public constant, so the cookie jar is effectively unprotected.
  Mock,
  /// Always use the operating system secret store, on every platform and in every build
  /// profile. Appends no switch at all.
  System,
}

/// What to do with Chromium's process sandbox.
///
/// Defaults to [`SandboxPolicy::Auto`], which keeps the sandbox on every platform except
/// in one situation: an application running from an AppImage on a Linux or BSD system
/// that offers no way to sandbox at all, where the alternative is not an unsandboxed
/// application but no application, since Chromium aborts with "No usable sandbox!".
///
/// # This used to be a cargo feature
///
/// The sandbox was previously selected by `tauri-runtime-cef`'s `sandbox` feature, which
/// was on by default but silently produced a *fully unsandboxed* Chromium on Windows and
/// macOS for any consumer building with `default-features = false`. It is a runtime
/// setting now so that the decision is visible, overridable, and the same one on every
/// platform. The underlying sandbox support is always compiled in.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SandboxPolicy {
  /// Keep the sandbox, except when the application runs from an AppImage and the system
  /// has neither the setuid `chrome-sandbox` helper nor usable unprivileged user
  /// namespaces. A warning naming the reason is logged whenever the sandbox is dropped.
  ///
  /// That exception is Linux and BSD only. Windows and macOS link their sandbox into the
  /// executable rather than relying on a helper the system has to provide, so there is
  /// nothing that can be missing and `Auto` there is the same as [`Self::Required`].
  #[default]
  Auto,
  /// Never run without a sandbox, even when that means Chromium aborts at startup.
  ///
  /// Pick this when running unsandboxed is not an acceptable outcome and a hard failure
  /// is preferable — the user can then install the setuid helper, point
  /// `CHROME_DEVEL_SANDBOX` at one, or re-enable unprivileged user namespaces.
  Required,
  /// Always run without a sandbox, on every platform.
  ///
  /// Every renderer then runs with the full privileges of the user, so a compromised
  /// renderer is a compromised account. Useful for containers and CI images that cannot
  /// provide a sandbox, not for shipped applications.
  Disabled,
}

/// Selects and configures the CEF runtime.
///
/// Pass it to `tauri::Builder::runtime` to run the application with CEF:
///
/// ```rust,no_run
/// tauri::Builder::default().runtime(
///   tauri_runtime_cef::Cef::default().command_line_arg("disable-gpu", None::<String>),
/// );
/// ```
#[derive(Default)]
pub struct Cef {
  command_line_args: Vec<(String, Option<String>)>,
  deep_link_schemes: Vec<String>,
  cache_path: Option<PathBuf>,
  api_version: Option<i32>,
  secret_storage: SecretStorage,
  profile_preferences: Vec<(String, bool)>,
  allow_chromium_command_line_args: bool,
  log_file: Option<PathBuf>,
  log_severity: Option<LogSeverity>,
  locale: Option<String>,
  accept_language_list: Option<String>,
  sandbox: SandboxPolicy,
  settings_callback: Option<Box<SettingsCallback>>,
}

impl fmt::Debug for Cef {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("Cef")
      .field("command_line_args", &self.command_line_args)
      .field("deep_link_schemes", &self.deep_link_schemes)
      .field("cache_path", &self.cache_path)
      .field("api_version", &self.api_version)
      .field("secret_storage", &self.secret_storage)
      .field("profile_preferences", &self.profile_preferences)
      .field(
        "allow_chromium_command_line_args",
        &self.allow_chromium_command_line_args,
      )
      .field("log_file", &self.log_file)
      .field("log_severity", &self.log_severity)
      .field("locale", &self.locale)
      .field("accept_language_list", &self.accept_language_list)
      .field("sandbox", &self.sandbox)
      .field("settings_callback", &self.settings_callback.is_some())
      .finish()
  }
}

impl Cef {
  /// Sets a callback to customize the settings passed to [`cef::initialize`].
  ///
  /// If called more than once, only the last callback is used.
  #[must_use]
  pub fn with_settings<F>(mut self, callback: F) -> Self
  where
    F: FnOnce(&mut cef::Settings) + Send + Sync + 'static,
  {
    self.settings_callback = Some(Box::new(callback));
    self
  }

  /// Appends one command line argument passed to CEF.
  ///
  /// The argument is applied to the **browser process only**. CEF warns that modifying
  /// the command line of a non-browser process "may result in undefined behavior
  /// including crashes", and Chromium already forwards to each child process the
  /// switches it needs.
  #[must_use]
  pub fn command_line_arg<K: Into<String>, V: Into<String>>(
    mut self,
    key: K,
    value: Option<V>,
  ) -> Self {
    self
      .command_line_args
      .push((key.into(), value.map(Into::into)));
    self
  }

  /// Appends a list of command line arguments passed to CEF.
  ///
  /// Like [`Self::command_line_arg`], these are applied to the browser process only.
  #[must_use]
  pub fn command_line_args<K: Into<String>, V: Into<String>>(
    mut self,
    args: impl IntoIterator<Item = (K, Option<V>)>,
  ) -> Self {
    self
      .command_line_args
      .extend(args.into_iter().map(|(k, v)| (k.into(), v.map(Into::into))));
    self
  }

  /// Appends a list of deep link schemes detected by CEF's on_already_running_app_relaunch hook.
  ///
  /// Deep links defined by the core deep-link plugin on the Tauri configuration are automatically added.
  #[must_use]
  pub fn deep_link_schemes<S: Into<String>>(
    mut self,
    schemes: impl IntoIterator<Item = S>,
  ) -> Self {
    self
      .deep_link_schemes
      .extend(schemes.into_iter().map(Into::into));
    self
  }

  /// Directory used for CEF disk cache (`Settings::cache_path`).
  ///
  /// If unspecified, defaults to `{user cache}/{app identifier}/cef`.
  #[must_use]
  pub fn root_cache_path<P: AsRef<std::path::Path>>(mut self, path: P) -> Self {
    self.cache_path = Some(path.as_ref().to_path_buf());
    self
  }

  /// CEF API version this process declares (`cef_api_hash`), defaulting to
  /// `cef::sys::CEF_API_VERSION_LAST`.
  #[must_use]
  pub fn cef_api_version(mut self, version: i32) -> Self {
    self.api_version = Some(version);
    self
  }

  /// Which key Chromium uses to encrypt cookies and saved passwords at rest.
  ///
  /// Defaults to [`SecretStorage::Auto`]: development builds skip the OS secret store —
  /// the macOS keychain prompt is replaced by a mock keychain, and Linux skips the D-Bus
  /// secret portal, libsecret and KWallet — while release builds use it. Windows keeps
  /// using DPAPI throughout.
  ///
  /// Nothing but cookies and saved passwords is affected — `localStorage` and IndexedDB
  /// are stored unencrypted whichever variant you choose. The mock keychain and the
  /// Linux `basic` store encrypt with a hard-coded, publicly known constant, so they
  /// offer no meaningful protection at rest; reach for [`SecretStorage::Mock`] only when
  /// a release build has to run where no secret store exists at all, such as a container
  /// or a headless session. Because development and release builds share the default
  /// cache directory, switching between key sources makes previously stored cookies
  /// unreadable. See [`SecretStorage`] for the details.
  #[must_use]
  pub fn secret_storage(mut self, storage: SecretStorage) -> Self {
    self.secret_storage = storage;
    self
  }

  /// Sets one boolean Chromium profile preference on every webview's request context.
  ///
  /// Applied after the runtime's own defaults, so it can turn a preference the runtime
  /// disabled back on as well as turn something else off. Calling it twice for the same
  /// preference keeps the last value.
  ///
  /// The runtime disables a handful of Chrome browser features that have no place in an
  /// application webview — the "Save password?" and address and credit-card bubbles
  /// (`credentials_enable_service`, `profile.password_manager_leak_detection`,
  /// `autofill.profile_enabled`, `autofill.credit_card_enabled`), the translate bubble
  /// (`translate.enabled`), and two background requests to Google
  /// (`alternate_error_pages.enabled`, `search.suggest_enabled`). An application that
  /// wants one of them, such as a browser-shaped app that really does want the password
  /// manager, names it here.
  ///
  /// Safe Browsing (`safebrowsing.enabled`) is left on by the runtime and is the other
  /// reason this exists: an application whose webview only ever loads its own content can
  /// switch it off here.
  ///
  /// Preference names are Chromium's own, and which ones a given Chrome build registers
  /// as writable varies. A preference this build refuses is logged at debug and skipped.
  ///
  /// ```no_run
  /// # use tauri_runtime_cef::Cef;
  /// Cef::default()
  ///   .profile_preference("credentials_enable_service", true)
  ///   .profile_preference("safebrowsing.enabled", false);
  /// ```
  #[must_use]
  pub fn profile_preference<K: Into<String>>(mut self, name: K, enabled: bool) -> Self {
    self.profile_preferences.push((name.into(), enabled));
    self
  }

  /// Lets Chromium read switches off the process command line in release builds.
  ///
  /// Release builds ignore them by default (`Settings::command_line_args_disabled`),
  /// because otherwise anyone who can start the shipped executable can also start it
  /// with `--remote-debugging-port` and drive the app over the DevTools protocol, or
  /// with `--disable-web-security`, `--proxy-server`, `--host-resolver-rules` or
  /// `--ssl-key-log-file` — Chromium honours every one of them. Development builds
  /// (`tauri::is_dev()`) always keep the command line enabled.
  ///
  /// Enable this only if the application genuinely needs users to pass Chromium
  /// switches, and note that it is not a complete lockdown either way: the network
  /// service reads the `SSLKEYLOGFILE` environment variable regardless of this setting,
  /// so TLS session keys can still be dumped by an attacker who controls the
  /// application's environment.
  ///
  /// Switches configured through [`Self::command_line_arg`] are unaffected: CEF clears
  /// Chromium's command line before applying its own settings and before calling the
  /// runtime's `on_before_command_line_processing` hook. Tauri's own CLI parsing and its
  /// cold-start deep link handling read `std::env::args()`, which Chromium never
  /// touches, and are unaffected too.
  ///
  /// Deep links delivered to an *already running* instance do go through Chromium: its
  /// process singleton relays the second process's command line, which CEF has by then
  /// cleared. The runtime restores the deep link URL onto that command line so
  /// `myapp://...` still reaches the running application either way.
  #[must_use]
  pub fn allow_chromium_command_line_args(mut self, allow: bool) -> Self {
    self.allow_chromium_command_line_args = allow;
    self
  }

  /// File Chromium and CEF write their log to (`Settings::log_file`).
  ///
  /// Defaults to `cef.log` inside the cache directory (see [`Self::root_cache_path`]).
  /// With no log file configured, CEF writes a `debug.log` into the *main executable
  /// directory* on Windows and Linux, which for an installed application is a location
  /// the user did not expect a file to appear in and often cannot write to at all.
  ///
  /// Note that the default also overrides the macOS convention, where CEF would
  /// otherwise write to `~/Library/Logs/<app name>_debug.log`. Pass that path explicitly
  /// to keep it.
  #[must_use]
  pub fn log_file<P: AsRef<std::path::Path>>(mut self, path: P) -> Self {
    self.log_file = Some(path.as_ref().to_path_buf());
    self
  }

  /// Lowest severity Chromium and CEF write to the log file (`Settings::log_severity`).
  ///
  /// Defaults to [`cef::LogSeverity::WARNING`] in release builds — CEF's own default is
  /// `INFO`, which is chatty enough to grow the log file of a long-running application —
  /// and to [`cef::LogSeverity::DEFAULT`] in development builds (`tauri::is_dev()`),
  /// where the informational messages are usually what you want.
  ///
  /// [`cef::LogSeverity::DISABLE`] does not turn logging off entirely: CEF maps it to a
  /// FATAL-only minimum level, so nothing is written to the log file but FATAL messages
  /// still go to stderr.
  #[must_use]
  pub fn log_severity(mut self, severity: LogSeverity) -> Self {
    self.log_severity = Some(severity);
    self
  }

  /// Locale Chromium loads its own localized resources for (`Settings::locale`),
  /// as an ISO language code such as `en-US` or `pt-BR`.
  ///
  /// Leave unset — the default — unless you know the matching pak file ships with the
  /// application. Tauri's bundler currently packages **only the `en-US` locale pak**, so
  /// naming any other locale leaves Chromium unable to load the localized strings it
  /// uses for its own UI (context menus, error pages, form controls). This does not
  /// affect the application's own content, nor which languages a website is asked for —
  /// that is [`Self::accept_language_list`].
  #[must_use]
  pub fn locale<S: Into<String>>(mut self, locale: S) -> Self {
    self.locale = Some(locale.into());
    self
  }

  /// Comma-delimited list of languages sent as the `Accept-Language` header and reported
  /// through `navigator.language` (`Settings::accept_language_list`), for example
  /// `en-US,en,pt-BR`.
  ///
  /// Defaults to CEF's own value, which is derived from [`Self::locale`].
  #[must_use]
  pub fn accept_language_list<S: Into<String>>(mut self, languages: S) -> Self {
    self.accept_language_list = Some(languages.into());
    self
  }

  /// What to do with Chromium's process sandbox.
  ///
  /// Defaults to [`SandboxPolicy::Auto`], which keeps the sandbox except when the
  /// application runs from an AppImage on a Linux or BSD system that offers no way to
  /// sandbox at all — AppImages cannot ship the setuid `chrome-sandbox` helper the deb
  /// and rpm bundlers install, and distributions such as Ubuntu 23.10 and later restrict
  /// the unprivileged user namespaces Chromium would otherwise fall back to. Without the
  /// escape hatch Chromium aborts at startup with "No usable sandbox!".
  ///
  /// See [`SandboxPolicy`] for the other variants. This replaces the crate's former
  /// `sandbox` cargo feature, which could silently drop the sandbox on Windows and macOS.
  #[must_use]
  pub fn sandbox(mut self, policy: SandboxPolicy) -> Self {
    self.sandbox = policy;
    self
  }
}

impl<T: UserEvent> tauri_runtime::RuntimeInitAttrs<T> for Cef {
  type Runtime = CefRuntime<T>;

  fn apply_config(&mut self, config: &tauri_utils::config::Config) -> Result<()> {
    if let Some(plugin_config) = config
      .plugins
      .0
      .get("deep-link")
      .and_then(|config| config.get("desktop").cloned())
    {
      #[derive(serde::Deserialize)]
      #[serde(untagged)]
      enum DesktopDeepLinks {
        One(tauri_utils::config::DeepLinkProtocol),
        List(Vec<tauri_utils::config::DeepLinkProtocol>),
      }

      let protocols: DesktopDeepLinks =
        serde_json::from_value(plugin_config).map_err(tauri_runtime::Error::Json)?;
      let schemes = match protocols {
        DesktopDeepLinks::One(protocol) => protocol.schemes,
        DesktopDeepLinks::List(protocols) => protocols
          .into_iter()
          .flat_map(|protocol| protocol.schemes)
          .collect(),
      };

      self.deep_link_schemes.extend(schemes);
    }
    Ok(())
  }
}

impl<T: UserEvent> From<Cef> for tauri_runtime::dynamic::DynRuntimeInitAttrs<T> {
  fn from(attrs: Cef) -> Self {
    Self::new(attrs)
  }
}

/// Information about the CEF webview that requested a new window.
pub struct NewWindowOpener {
  source_url: Option<url::Url>,
}

impl NewWindowOpener {
  pub(crate) fn new(source_url: Option<url::Url>) -> Self {
    Self { source_url }
  }

  /// The opener's main-frame URL at the native popup request, when available.
  ///
  /// CEF supplies this directly from the callback's browser. Reading a blocking
  /// webview getter from that callback can deadlock the UI thread because CEF's
  /// external message pump may run outside a winit dispatch callback.
  pub fn source_url(&self) -> Option<&url::Url> {
    self.source_url.as_ref()
  }
}

impl std::fmt::Debug for NewWindowOpener {
  fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    // The URL can carry credentials and tokens, so only its presence is shown.
    formatter
      .debug_struct("NewWindowOpener")
      .field("source_url_observed", &self.source_url.is_some())
      .finish()
  }
}

#[derive(Clone, Debug)]
pub struct EventProxy<T: UserEvent> {
  context: RuntimeContext<T>,
}

impl<T: UserEvent> EventLoopProxy<T> for EventProxy<T> {
  fn send_event(&self, event: T) -> Result<()> {
    self.context.send_message(Message::UserEvent(event))
  }
}

#[derive(Clone)]
pub(crate) struct RuntimeContext<T: UserEvent> {
  pub(crate) sender: Sender<Message<T>>,
  pub(crate) proxy: WinitEventLoopProxy,
  main_thread_id: std::thread::ThreadId,
  next_window_id: Arc<AtomicU32>,
  next_webview_id: Arc<AtomicU32>,
  next_window_event_id: Arc<AtomicU32>,
  next_webview_event_id: Arc<AtomicU32>,
  current_dispatch: Arc<MainThreadDispatchSlot<T>>,
  pub(crate) app_wide_theme: Arc<Mutex<Option<Theme>>>,
  pub(crate) cef_pump: CefExternalPump,
  /// Root cache path passed to [`cef::Settings::cache_path`] during
  /// [`cef::initialize`]. Per-webview `data_directory` profiles must resolve
  /// under this root for CEF request contexts to be accepted.
  pub(crate) cache_path: Arc<PathBuf>,
  /// Chromium profile preferences the application asked for, applied to every
  /// webview's request context after the runtime's own defaults. See
  /// [`Cef::profile_preference`].
  pub(crate) profile_preferences: Arc<Vec<(String, bool)>>,
}

/// Scoped access to the current winit callback state.
///
/// `ActiveEventLoop` is only borrowed during `ApplicationHandler` callbacks, but
/// setup-time runtime messages may synchronously need it. While a callback is
/// active, this slot lets main-thread `send_message` handle work immediately;
/// other threads still queue and wake the loop. The slot stores an atomic
/// pointer to guard-owned state so lookup is lock-free. The guard restores the
/// previous pointer before dropping that state, so the raw pointers are never
/// treated as valid beyond their callback.
#[derive(Clone, Copy)]
struct MainThreadDispatch<T: UserEvent> {
  app: *mut WinitCefApp<T>,
  event_loop: *const dyn ActiveEventLoop,
}

struct MainThreadDispatchSlot<T: UserEvent> {
  current: AtomicPtr<MainThreadDispatch<T>>,
}

impl<T: UserEvent> MainThreadDispatchSlot<T> {
  fn install(&self, dispatch: &mut MainThreadDispatch<T>) -> *mut MainThreadDispatch<T> {
    self.current.swap(dispatch, Ordering::AcqRel)
  }

  fn restore(&self, current: *mut MainThreadDispatch<T>, previous: *mut MainThreadDispatch<T>) {
    let installed = self.current.swap(previous, Ordering::AcqRel);
    debug_assert_eq!(installed, current);
  }

  fn current(&self) -> Option<&MainThreadDispatch<T>> {
    let current = self.current.load(Ordering::Acquire);
    if current.is_null() {
      None
    } else {
      // SAFETY: the pointer targets the boxed dispatch state owned by
      // `MainThreadDispatchGuard`, whose allocation remains stable while the
      // guard is moved. The slot is restored before that state is dropped, and
      // it is only read by `send_message` after verifying that it is running on
      // the runtime main thread.
      Some(unsafe { &*current })
    }
  }
}

impl<T: UserEvent> Default for MainThreadDispatchSlot<T> {
  fn default() -> Self {
    Self {
      current: AtomicPtr::new(std::ptr::null_mut()),
    }
  }
}

struct MainThreadDispatchGuard<T: UserEvent> {
  context: RuntimeContext<T>,
  dispatch: Box<MainThreadDispatch<T>>,
  previous: *mut MainThreadDispatch<T>,
}

impl<T: UserEvent> Drop for MainThreadDispatchGuard<T> {
  fn drop(&mut self) {
    self
      .context
      .current_dispatch
      .restore(self.dispatch.as_mut(), self.previous);
  }
}

#[allow(clippy::result_large_err)]
fn handle_main_thread_message<T: UserEvent>(
  context: &RuntimeContext<T>,
  message: Message<T>,
) -> std::result::Result<(), Message<T>> {
  let Some(dispatch) = context.current_dispatch.current() else {
    return Err(message);
  };

  // SAFETY: `WinitCefApp::install_current_dispatch` stores pointers to the currently
  // executing winit application handler and event-loop callback. This function
  // is only called on the runtime main thread while that callback is active.
  let app = unsafe { &mut *dispatch.app };
  let event_loop = unsafe { &*dispatch.event_loop };

  app.handle_message(event_loop, message);

  Ok(())
}

impl<T: UserEvent> fmt::Debug for RuntimeContext<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("RuntimeContext").finish()
  }
}

impl<T: UserEvent> RuntimeContext<T> {
  pub(crate) fn send_message(&self, message: Message<T>) -> Result<()> {
    let message = if self.is_main_thread() {
      match handle_main_thread_message(self, message) {
        Ok(()) => return Ok(()),
        Err(message) => message,
      }
    } else {
      message
    };

    self
      .sender
      .send(message)
      .map_err(|_| Error::FailedToSendMessage)?;
    self.proxy.wake_up();
    Ok(())
  }

  pub(crate) fn is_main_thread(&self) -> bool {
    std::thread::current().id() == self.main_thread_id
  }

  /// Run `f` on the main (event-loop) thread.
  ///
  /// When called from the main thread we execute `f` inline instead of posting
  /// it to the channel. Tauri implements several blocking getters as
  /// `run_on_main_thread(|| { .. tx.send(..) }); rx.recv()` (e.g.
  /// `Window::add_child`). Those run during `setup`, which the runtime drives
  /// from winit's `can_create_surfaces`; posting the closure instead of running
  /// it inline would deadlock because the loop cannot drain the task while the
  /// main thread blocks on `rx.recv()`.
  pub(crate) fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> Result<()> {
    if self.is_main_thread() {
      f();
      Ok(())
    } else {
      self.send_message(Message::Task(Box::new(f)))
    }
  }

  pub(crate) fn next_window_id(&self) -> WindowId {
    self.next_window_id.fetch_add(1, Ordering::Relaxed).into()
  }

  pub(crate) fn next_webview_id(&self) -> u32 {
    self.next_webview_id.fetch_add(1, Ordering::Relaxed)
  }

  pub(crate) fn next_window_event_id(&self) -> u32 {
    self.next_window_event_id.fetch_add(1, Ordering::Relaxed)
  }

  pub(crate) fn next_webview_event_id(&self) -> u32 {
    self.next_webview_event_id.fetch_add(1, Ordering::Relaxed)
  }
}

pub(crate) type AfterWindowCreationCallback = Box<dyn for<'a> Fn(RawWindow<'a>) + Send>;

pub(crate) enum Message<T: UserEvent> {
  EventLoop(EventLoopMessage),
  BrowserClosed(WindowId, u32),
  PopupPending(crate::popup::PopupRequest, Arc<crate::popup::PopupFamily>),
  PopupCreated(
    crate::popup::PopupRequest,
    i32,
    Arc<crate::popup::PopupFamily>,
  ),
  PopupAborted(crate::popup::PopupRequest),
  PopupClosed(i32),
  /// CEF handed us the teardown of a webview's browser, keyed by the webview's
  /// process-unique id. See `TauriCefChildLifeSpanHandler::do_close`.
  #[cfg(any(target_os = "macos", windows))]
  DestroyWebviewHostWindow(u32),
  Opened(Vec<url::Url>),
  #[cfg(target_os = "macos")]
  Reopen {
    has_visible_windows: bool,
  },
  #[cfg(target_os = "macos")]
  AccessibilityChanged {
    enabled: bool,
  },
  CreateWindow {
    window_id: WindowId,
    webview_id: Option<u32>,
    pending: Box<PendingWindow<T, CefRuntime<T>>>,
    after_window_creation: Option<AfterWindowCreationCallback>,
    result_tx: Sender<Result<()>>,
  },
  CreateWebview {
    window_id: WindowId,
    webview_id: u32,
    pending: Box<PendingWebview<T, CefRuntime<T>>>,
    result_tx: Sender<Result<()>>,
  },
  Window {
    window_id: WindowId,
    message: WindowMessage,
  },
  Webview {
    window_id: WindowId,
    webview_id: u32,
    message: WebviewMessage,
  },
  NavigateFirstWebview {
    window_id: WindowId,
    url: String,
  },
  DragDropScriptEvent {
    window_id: WindowId,
    webview_id: u32,
    target: browser_client::DragDropEventTarget,
    drag_drop_state: Arc<Mutex<browser_client::DragDropState>>,
    event: browser_client::DragDropScriptEvent,
  },
  Task(Box<dyn FnOnce() + Send>),
  RequestExit(i32),
  UserEvent(T),
}

fn device_event_filter_to_winit(filter: DeviceEventFilter) -> winit::event_loop::DeviceEvents {
  match filter {
    DeviceEventFilter::Always => winit::event_loop::DeviceEvents::Never,
    DeviceEventFilter::Unfocused => winit::event_loop::DeviceEvents::WhenFocused,
    DeviceEventFilter::Never => winit::event_loop::DeviceEvents::Always,
  }
}

pub(crate) enum EventLoopMessage {
  SetTheme(Option<Theme>),
  SetDeviceEventFilter(DeviceEventFilter),
  PrimaryMonitor(Sender<Result<Option<Monitor>>>),
  MonitorFromPoint(Sender<Result<Option<Monitor>>>, f64, f64),
  AvailableMonitors(Sender<Result<Vec<Monitor>>>),
  CursorPosition(Sender<Result<PhysicalPosition<f64>>>),
  DisplayHandle(Sender<std::result::Result<SendRawDisplayHandle, raw_window_handle::HandleError>>),
  #[cfg(target_os = "macos")]
  SetActivationPolicy(tauri_runtime::ActivationPolicy),
  #[cfg(target_os = "macos")]
  SetDockVisibility(bool),
  #[cfg(target_os = "macos")]
  ShowApplication,
  #[cfg(target_os = "macos")]
  HideApplication,
}

macro_rules! event_loop_getter {
  ($self:ident, $variant:ident) => {{
    let (tx, rx) = mpsc::channel();
    match $self
      .context
      .send_message(Message::EventLoop(EventLoopMessage::$variant(tx)))
    {
      Ok(()) => rx.recv().map_err(|_| Error::FailedToReceiveMessage),
      Err(error) => Err(error),
    }
  }};
}

fn find_monitor_from_point(
  monitors: impl Iterator<Item = winit::monitor::MonitorHandle>,
  x: f64,
  y: f64,
) -> Option<winit::monitor::MonitorHandle> {
  monitors.into_iter().find(|monitor| {
    let pos = monitor.position().unwrap_or_default();
    let size = monitor
      .current_video_mode()
      .map(|mode| mode.size())
      .unwrap_or_default();
    x >= pos.x as f64
      && x <= pos.x as f64 + size.width as f64
      && y >= pos.y as f64
      && y <= pos.y as f64 + size.height as f64
  })
}

#[cfg(target_os = "macos")]
fn is_cef_helper_process() -> bool {
  const HELPER_SUFFIXES: &[&str] = &[
    " Helper (GPU)",
    " Helper (Renderer)",
    " Helper (Plugin)",
    " Helper (Alerts)",
    " Helper",
  ];

  std::env::current_exe()
    .ok()
    .and_then(|path| {
      path
        .file_name()
        .and_then(|name| name.to_str())
        .map(|name| HELPER_SUFFIXES.iter().any(|suffix| name.ends_with(suffix)))
    })
    .unwrap_or_default()
}

pub(crate) struct AppState<T: UserEvent> {
  pub(crate) windows: HashMap<WindowId, AppWindow>,
  pub(crate) winid_id_to_window_id_map: HashMap<WinitWindowId, WindowId>,
  pub(crate) callback: Box<dyn FnMut(RunEvent<T>)>,
  pub(crate) live_browsers: usize,
  live_popups: HashMap<i32, Arc<crate::popup::PopupFamily>>,
  pending_popups: Vec<(crate::popup::PopupRequest, Arc<crate::popup::PopupFamily>)>,
  pub(crate) exiting: bool,
}

pub(crate) struct WinitCefApp<T: UserEvent> {
  pub(crate) context: RuntimeContext<T>,
  receiver: Receiver<Message<T>>,
  pub(crate) state: AppState<T>,
  pub(crate) scheme_registry: request_handler::SchemeRegistry,
}

impl<T: UserEvent> WinitCefApp<T> {
  fn new(
    context: RuntimeContext<T>,
    receiver: Receiver<Message<T>>,
    callback: Box<dyn FnMut(RunEvent<T>)>,
    scheme_registry: request_handler::SchemeRegistry,
  ) -> Self {
    Self {
      context,
      receiver,
      state: AppState {
        windows: HashMap::new(),
        winid_id_to_window_id_map: HashMap::new(),
        callback,
        live_browsers: 0,
        live_popups: HashMap::new(),
        pending_popups: Vec::new(),
        exiting: false,
      },
      scheme_registry,
    }
  }

  fn run_callback(&mut self, event: RunEvent<T>) {
    (self.state.callback)(event);
  }

  fn install_current_dispatch(
    &mut self,
    event_loop: &dyn ActiveEventLoop,
  ) -> MainThreadDispatchGuard<T> {
    let mut dispatch = Box::new(MainThreadDispatch {
      app: self as *mut _,
      event_loop: event_loop as *const _,
    });

    let previous = self.context.current_dispatch.install(dispatch.as_mut());

    MainThreadDispatchGuard {
      context: self.context.clone(),
      dispatch,
      previous,
    }
  }

  fn drain_messages(&mut self, event_loop: &dyn ActiveEventLoop) {
    while let Ok(message) = self.receiver.try_recv() {
      self.handle_message(event_loop, message);
    }
  }

  fn handle_message(&mut self, event_loop: &dyn ActiveEventLoop, message: Message<T>) {
    match message {
      Message::EventLoop(message) => self.handle_event_loop_message(event_loop, message),
      Message::PopupPending(request, family) => {
        self.state.pending_popups.push((request, family));
      }
      Message::PopupCreated(request, id, family) => {
        self
          .state
          .pending_popups
          .retain(|(pending, _)| !pending.is_same(&request));
        self.state.live_popups.insert(id, family);
      }
      Message::PopupAborted(request) => {
        self
          .state
          .pending_popups
          .retain(|(pending, _)| !pending.is_same(&request));
        self.exit_if_done(event_loop);
      }
      Message::PopupClosed(id) => {
        self.state.live_popups.remove(&id);
        self.exit_if_done(event_loop);
      }
      Message::BrowserClosed(_window_id, webview_id) => {
        // Standalone webview.close() and app shutdown keep the child in state
        // until this callback, so cleanup happens here. Individual window
        // teardown removes child bookkeeping first; then this message is only
        // the lifecycle acknowledgement that lets live_browsers drain.
        //
        // The window_id baked into the browser's handlers can be stale after a
        // reparent, so locate the webview by its process-unique id across every
        // window rather than trusting the message's window_id — otherwise a
        // reparented webview's scheme-handler entries would leak and its
        // AppWebview would linger in the target window forever.
        let closed = self.state.windows.iter_mut().find_map(|(id, appwindow)| {
          appwindow
            .children
            .iter()
            .position(|child| child.webview_id == webview_id)
            .map(|index| {
              let child = appwindow.children.remove(index);
              (*id, child, appwindow.children.is_empty())
            })
        });

        let mut emptied_window = None;
        if let Some((window_id, child, was_last)) = closed {
          self.remove_scheme_handler_entries(&child);
          if was_last {
            emptied_window = Some(window_id);
          }
        }

        self.state.live_browsers = self.state.live_browsers.saturating_sub(1);

        // A window that just lost its last webview has nothing left to show, so
        // it follows the webview out through the regular close path — listeners
        // still get `CloseRequested` and can keep the empty window around.
        // `close_window` runs the exit check itself.
        if let Some(window_id) = emptied_window {
          self.request_window_close(window_id, event_loop);
        } else {
          self.exit_if_done(event_loop);
        }
      }
      #[cfg(any(target_os = "macos", windows))]
      Message::DestroyWebviewHostWindow(webview_id) => {
        // Destroying the browser's own child view/window is what completes the
        // close CEF handed over in `do_close`; CEF acknowledges it with
        // `BrowserClosed`, which is where the bookkeeping is dropped. Same
        // reasoning as there for searching every window by webview id.
        //
        // A webview that is already gone from state means its window is being
        // torn down, and that teardown destroys the child view anyway.
        if let Some(child) = self
          .state
          .windows
          .values()
          .flat_map(|appwindow| appwindow.children.iter())
          .find(|child| child.webview_id == webview_id)
        {
          child.destroy_host_window();
        }
      }
      Message::CreateWindow {
        window_id,
        webview_id,
        pending,
        after_window_creation,
        result_tx,
      } => {
        let result = self.create_window(
          event_loop,
          window_id,
          webview_id,
          pending,
          after_window_creation,
        );
        let _ = result_tx.send(result);
      }
      Message::CreateWebview {
        window_id,
        webview_id,
        pending,
        result_tx,
      } => {
        let _ = result_tx.send(self.create_webview(window_id, webview_id, *pending));
      }
      Message::Window { window_id, message } => {
        self.handle_window_message(event_loop, window_id, message)
      }
      Message::Webview {
        window_id,
        webview_id,
        message,
      } => self.handle_webview_message(window_id, webview_id, message),
      Message::NavigateFirstWebview { window_id, url } => {
        self.navigate_first_webview(window_id, &url)
      }
      Message::DragDropScriptEvent {
        window_id,
        webview_id,
        target,
        drag_drop_state,
        event,
      } => {
        if let Some(event) = browser_client::event_from_script_event(&drag_drop_state, event) {
          self.emit_drag_drop_event(window_id, webview_id, target, event);
        }
      }
      Message::Task(task) => task(),
      Message::RequestExit(code) => {
        if self.request_exit(Some(code)) {
          self.close_all_browsers();
          self.exit_if_done(event_loop);
        }
      }
      Message::Opened(urls) => self.run_callback(RunEvent::Opened { urls }),
      #[cfg(target_os = "macos")]
      Message::Reopen {
        has_visible_windows,
      } => self.run_callback(RunEvent::Reopen {
        has_visible_windows,
      }),
      #[cfg(target_os = "macos")]
      Message::AccessibilityChanged { enabled } => self.set_browsers_accessibility_state(enabled),
      Message::UserEvent(event) => self.run_callback(RunEvent::UserEvent(event)),
    }
  }

  fn handle_event_loop_message(
    &mut self,
    event_loop: &dyn ActiveEventLoop,
    message: EventLoopMessage,
  ) {
    match message {
      EventLoopMessage::SetTheme(theme) => {
        *self.context.app_wide_theme.lock().unwrap() = theme;
        for appwindow in self.state.windows.values_mut() {
          appwindow.set_theme(theme);
        }
      }
      EventLoopMessage::PrimaryMonitor(tx) => {
        let monitor = event_loop
          .primary_monitor()
          .map(|monitor| winit_monitor_to_tauri_monitor(&monitor));
        let _ = tx.send(Ok(monitor));
      }
      EventLoopMessage::MonitorFromPoint(tx, x, y) => {
        let monitor = find_monitor_from_point(event_loop.available_monitors(), x, y)
          .map(|monitor| winit_monitor_to_tauri_monitor(&monitor));
        let _ = tx.send(Ok(monitor));
      }
      EventLoopMessage::AvailableMonitors(tx) => {
        let monitors = event_loop
          .available_monitors()
          .map(|monitor| winit_monitor_to_tauri_monitor(&monitor))
          .collect();
        let _ = tx.send(Ok(monitors));
      }
      EventLoopMessage::SetDeviceEventFilter(filter) => {
        event_loop.listen_device_events(device_event_filter_to_winit(filter));
      }
      EventLoopMessage::CursorPosition(tx) => {
        let _ = tx.send(event_loop.cursor_position());
      }
      EventLoopMessage::DisplayHandle(tx) => {
        let handle = event_loop
          .display_handle()
          .map(|handle| SendRawDisplayHandle(handle.as_raw()));
        let _ = tx.send(handle);
      }
      #[cfg(target_os = "macos")]
      EventLoopMessage::SetActivationPolicy(activation_policy) => {
        event_loop.set_activation_policy(activation_policy)
      }
      #[cfg(target_os = "macos")]
      EventLoopMessage::SetDockVisibility(visible) => event_loop.set_dock_visibility(visible),
      #[cfg(target_os = "macos")]
      EventLoopMessage::ShowApplication => event_loop.show_application(),
      #[cfg(target_os = "macos")]
      EventLoopMessage::HideApplication => event_loop.hide_application(),
    }
  }

  /// Removes the webview's `(browser_id, scheme)` entries from the scheme registry.
  fn remove_scheme_handler_entries(&self, child: &AppWebview) {
    let mut registry = self.scheme_registry.lock().unwrap();
    for scheme in child.uri_scheme_protocols.keys() {
      registry.remove(&(child.browser_id, scheme.clone()));
    }
  }

  fn emit_drag_drop_event(
    &mut self,
    window_id: WindowId,
    webview_id: u32,
    target: browser_client::DragDropEventTarget,
    event: DragDropEvent,
  ) {
    match target {
      browser_client::DragDropEventTarget::Window => {
        self.emit_window_event(window_id, WindowEvent::DragDrop(event));
      }
      browser_client::DragDropEventTarget::Webview => {
        self.emit_webview_event(window_id, webview_id, WebviewEvent::DragDrop(event));
      }
    }
  }

  fn emit_window_event(&mut self, window_id: WindowId, event: WindowEvent) {
    let Some(appwindow) = self.state.windows.get(&window_id) else {
      return;
    };
    let label = appwindow.label.clone();
    let listeners = appwindow.listeners.clone();

    self.run_callback(RunEvent::WindowEvent {
      label,
      event: event.clone(),
    });

    {
      let listeners = listeners.lock().unwrap();
      for handler in listeners.values() {
        handler(&event);
      }
    }
  }

  fn emit_webview_event(&mut self, window_id: WindowId, webview_id: u32, event: WebviewEvent) {
    let Some(appwindow) = self.state.windows.get(&window_id) else {
      return;
    };
    let Some(child) = appwindow
      .children
      .iter()
      .find(|child| child.webview_id == webview_id)
    else {
      return;
    };
    let label = child.label.clone();
    let listeners = child.listeners.clone();

    self.run_callback(RunEvent::WebviewEvent {
      label,
      event: event.clone(),
    });

    {
      let listeners = listeners.lock().unwrap();
      for handler in listeners.values() {
        handler(&event);
      }
    }
  }

  fn request_exit(&mut self, code: Option<i32>) -> bool {
    // if we already exiting, don't request exit again
    if self.state.exiting {
      return false;
    }

    let (tx, rx) = mpsc::channel();
    self.run_callback(RunEvent::ExitRequested { code, tx });

    if matches!(rx.try_recv(), Ok(ExitRequestedEventAction::Prevent)) {
      false
    } else {
      self.state.exiting = true;
      true
    }
  }

  pub(crate) fn close_window(&mut self, window_id: WindowId, event_loop: &dyn ActiveEventLoop) {
    let Some(appwindow) = self.state.windows.remove(&window_id) else {
      return;
    };
    self
      .state
      .winid_id_to_window_id_map
      .remove(&appwindow.window.id());
    // The window is gone from state, so BrowserClosed will not find these
    // children later. Clean registry entries while we still hold them; the CEF
    // shutdown drain is still enforced by live_browsers.
    for child in &appwindow.children {
      self.remove_scheme_handler_entries(child);
      child.popup_family.close_all();
      child.host.close_browser(1);
    }
    self.exit_if_done(event_loop);
  }

  pub(crate) fn request_window_close(
    &mut self,
    window_id: WindowId,
    event_loop: &dyn ActiveEventLoop,
  ) {
    // Avoid requesting window close if we already exisitng
    if self.state.exiting {
      self.close_window(window_id, event_loop);
      return;
    }

    let (tx, rx) = mpsc::channel();
    let Some(appwindow) = self.state.windows.get(&window_id) else {
      return;
    };
    let label = appwindow.label.clone();
    let listeners = appwindow.listeners.clone();

    {
      let listeners = listeners.lock().unwrap();
      for handler in listeners.values() {
        handler(&WindowEvent::CloseRequested {
          signal_tx: tx.clone(),
        });
      }
    }

    self.run_callback(RunEvent::WindowEvent {
      label,
      event: WindowEvent::CloseRequested { signal_tx: tx },
    });

    if !matches!(rx.try_recv(), Ok(true)) {
      self.close_window(window_id, event_loop);
    }
  }

  fn navigate_first_webview(&self, window_id: WindowId, url: &str) {
    let Some(frame) = self
      .state
      .windows
      .get(&window_id)
      .and_then(|window| window.children.first())
      .and_then(|webview| webview.browser.main_frame())
    else {
      return;
    };

    frame.load_url(Some(&CefString::from(url)));
  }

  fn close_all_browsers(&mut self) {
    // Keep each child reachable until CEF acknowledges its close. On macOS and
    // Windows, do_close queues DestroyWebviewHostWindow, which needs this state
    // to destroy the native child view and trigger on_before_close. Dropping
    // the windows here can strand live_browsers and prevent process exit.
    for appwindow in self.state.windows.values() {
      for child in &appwindow.children {
        child.popup_family.close_all();
        child.host.close_dev_tools();
        child.host.close_browser(1);
      }
    }
  }

  #[cfg(target_os = "macos")]
  fn set_browsers_accessibility_state(&self, enabled: bool) {
    let state = if enabled {
      State::ENABLED
    } else {
      State::DISABLED
    };
    for appwindow in self.state.windows.values() {
      for child in &appwindow.children {
        child.host.set_accessibility_state(state);
      }
    }
  }

  fn exit_if_done(&mut self, event_loop: &dyn ActiveEventLoop) {
    // A reservation is normally resolved by `PopupCreated` or `PopupAborted`,
    // but CEF discards popups without always reporting the abort — the opener
    // can be torn down first, or the abort can arrive for a browser its opener
    // no longer matches. Teardown (window close, app shutdown, the root's own
    // native close) revokes the family, and a revoked family never admits a
    // popup again, so its reservations are dead and must not hold the process
    // open. Reservations of live families still gate the exit until CEF
    // resolves them.
    self
      .state
      .pending_popups
      .retain(|(_, family)| !family.is_revoked());

    if self.state.live_browsers != 0
      || !self.state.live_popups.is_empty()
      || !self.state.pending_popups.is_empty()
    {
      return;
    }

    if self.state.exiting || (self.state.windows.is_empty() && self.request_exit(None)) {
      self.run_callback(RunEvent::Exit);
      event_loop.exit();
    }
  }

  /// Service the default GLib main context so the external message pump's GLib
  /// source (and any GTK work CEF schedules) gets dispatched, then arm winit to
  /// wake when the next GLib pump deadline is due. Windows/macOS need no
  /// equivalent: their pump timers live on the native loop winit already runs.
  #[cfg(any(
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  fn service_glib(&self, event_loop: &dyn ActiveEventLoop) {
    let context = gtk::glib::MainContext::default();
    while context.pending() {
      context.iteration(false);
    }
    if let Some(deadline) = self.context.cef_pump.next_deadline() {
      event_loop.set_control_flow(winit::event_loop::ControlFlow::WaitUntil(deadline));
    }
  }
}

impl<T: UserEvent> ApplicationHandler for WinitCefApp<T> {
  fn can_create_surfaces(&mut self, event_loop: &dyn ActiveEventLoop) {
    let _guard = self.install_current_dispatch(event_loop);
    self.drain_messages(event_loop);
  }

  fn new_events(&mut self, event_loop: &dyn ActiveEventLoop, cause: StartCause) {
    let _guard = self.install_current_dispatch(event_loop);
    match cause {
      StartCause::Init => {
        self.run_callback(RunEvent::Ready);
        self.context.cef_pump.do_work();
      }
      // Match wry/tao, which emit `Resumed` on each `Poll` start cause.
      StartCause::Poll => self.run_callback(RunEvent::Resumed),
      _ => {}
    }
  }

  fn proxy_wake_up(&mut self, event_loop: &dyn ActiveEventLoop) {
    let _guard = self.install_current_dispatch(event_loop);
    self.drain_messages(event_loop);
  }

  fn about_to_wait(&mut self, event_loop: &dyn ActiveEventLoop) {
    let _guard = self.install_current_dispatch(event_loop);
    self.apply_pending_activations();
    // TODO: remove once migrated to winit-gtk4
    #[cfg(any(
      target_os = "linux",
      target_os = "dragonfly",
      target_os = "freebsd",
      target_os = "netbsd",
      target_os = "openbsd"
    ))]
    self.service_glib(event_loop);
    self.run_callback(RunEvent::MainEventsCleared);
  }

  fn window_event(
    &mut self,
    event_loop: &dyn ActiveEventLoop,
    winit_id: WinitWindowId,
    event: WinitWindowEvent,
  ) {
    let _guard = self.install_current_dispatch(event_loop);
    let Some(window_id) = self.state.winid_id_to_window_id_map.get(&winit_id).copied() else {
      return;
    };
    let Some(appwindow) = self.state.windows.get_mut(&window_id) else {
      return;
    };

    match event {
      WinitWindowEvent::CloseRequested => self.request_window_close(window_id, event_loop),

      WinitWindowEvent::Destroyed => {
        if !self.state.exiting {
          self.emit_window_event(window_id, WindowEvent::Destroyed);
        }
        self.close_window(window_id, event_loop);
      }
      WinitWindowEvent::SurfaceResized(size) => {
        webview::layout_app_window(appwindow);
        self.emit_window_event(window_id, WindowEvent::Resized(size));
      }
      WinitWindowEvent::ScaleFactorChanged {
        scale_factor,
        surface_size_writer,
      } => {
        let new_inner_size = surface_size_writer
          .surface_size()
          .unwrap_or_else(|_| appwindow.window.surface_size());
        webview::layout_app_window(appwindow);
        self.emit_window_event(
          window_id,
          WindowEvent::ScaleFactorChanged {
            scale_factor,
            new_inner_size,
          },
        );
      }
      WinitWindowEvent::Moved(pos) => {
        self.emit_window_event(
          window_id,
          WindowEvent::Moved(PhysicalPosition::new(pos.x, pos.y)),
        );
      }
      WinitWindowEvent::Focused(focused) => {
        self.emit_window_event(window_id, WindowEvent::Focused(focused));
      }
      WinitWindowEvent::ThemeChanged(theme) => {
        let system_theme = winit_theme_to_tauri_theme(theme);
        if let Some(explicit_theme) = appwindow.preferred_theme() {
          appwindow.set_theme(Some(explicit_theme));
        } else {
          // Following the system: the appearance changed without going through
          // `set_theme`, so the titlebar rebuild still has to be undone.
          #[cfg(target_os = "macos")]
          appwindow.reapply_traffic_light_position_after_appearance_change();
        }
        self.emit_window_event(window_id, WindowEvent::ThemeChanged(system_theme));
      }
      WinitWindowEvent::DragEntered { paths, position } => {
        let event = DragDropEvent::Enter { paths, position };
        self.emit_window_event(window_id, WindowEvent::DragDrop(event));
      }
      WinitWindowEvent::DragMoved { position } => {
        let event = DragDropEvent::Over { position };
        self.emit_window_event(window_id, WindowEvent::DragDrop(event));
      }
      WinitWindowEvent::DragDropped { paths, position } => {
        let event = DragDropEvent::Drop { paths, position };
        self.emit_window_event(window_id, WindowEvent::DragDrop(event));
      }
      WinitWindowEvent::DragLeft { .. } => {
        self.emit_window_event(window_id, WindowEvent::DragDrop(DragDropEvent::Leave));
      }
      #[cfg(windows)]
      WinitWindowEvent::RedrawRequested => {
        appwindow.draw_background_surface();
      }
      _ => {}
    }
  }
}

/// Picks the deep link URLs out of a process command line.
///
/// An argument qualifies when it parses as a URL whose scheme is one of
/// `schemes`; scheme matching is the same exact comparison
/// `BrowserProcessHandler::on_already_running_app_relaunch` performs on the
/// receiving end. Everything else is dropped: the whole point of
/// [`Cef::allow_chromium_command_line_args`] being off is that no other argument
/// survives onto Chromium's command line.
fn deep_link_arguments<I>(args: I, schemes: &[String]) -> Vec<String>
where
  I: IntoIterator<Item = String>,
{
  args
    .into_iter()
    .filter(|arg| {
      url::Url::parse(arg).is_ok_and(|url| schemes.iter().any(|scheme| scheme == url.scheme()))
    })
    .collect()
}

/// Appends `args` to `command_line`, as a switch with a value, a bare switch or a
/// positional argument depending on how each entry looks.
///
/// A bare name with no value is only recognised as a switch when it is spelled with its
/// `--` prefix; without one it is a positional argument. This runtime's own entries are
/// therefore all spelled `--switch`, values included: Chromium strips the prefix off the
/// key it stores (`CommandLine::AppendSwitchNative`), so both spellings reach the same
/// switch and one convention avoids having to remember which form each entry needs.
fn append_command_line_args(command_line: &mut CommandLine, args: &[(String, Option<String>)]) {
  for (arg, value) in args {
    if let Some(value) = value {
      command_line.append_switch_with_value(
        Some(&CefString::from(arg.as_str())),
        Some(&CefString::from(value.as_str())),
      );
    } else if arg.starts_with("-") {
      command_line.append_switch(Some(&CefString::from(arg.as_str())));
    } else {
      command_line.append_argument(Some(&CefString::from(arg.as_str())));
    }
  }
}

wrap_with_args! {
  wrap_app => TauriCefAppArgs;

  struct TauriCefApp<T: UserEvent> {
    context: RuntimeContext<T>,
    context_initialized: Arc<AtomicBool>,
    deep_link_schemes: Vec<String>,
    // Whether the deep link URL this process was launched with has to be put back
    // onto Chromium's command line. See `on_before_command_line_processing`.
    restore_deep_link_arguments: bool,
    // Switches applied whatever process type `on_before_command_line_processing` reports.
    //
    // Deliberately tiny: `cef_app_t::on_before_command_line_processing` warns that
    // "modifying the command-line arguments for non-browser processes may result in
    // undefined behavior including crashes", so only switches we know a child process
    // must see itself belong here.
    internal_command_line_args: Vec<(String, Option<String>)>,
    // Switches applied only when the reported process type is the browser one.
    //
    // Chromium already forwards to each child the switches it needs, so anything that
    // is only read in the browser process - and everything the embedding application
    // supplied through `Cef::command_line_arg` - goes here. The application's own
    // switches are appended last so they win over the runtime's defaults.
    //
    // In practice both lists only ever reach a browser process: `CefRuntime::init` sends
    // every subprocess into `TauriCefHelperApp` and exits before `TauriCefApp` is built,
    // so this app is never installed anywhere else and the process type it is handed is
    // always the empty, browser one. The split is a guard against that stopping to hold,
    // not a distinction that changes behaviour today.
    browser_command_line_args: Vec<(String, Option<String>)>,
  }

  impl App {
    fn render_process_handler(&self) -> Option<RenderProcessHandler> {
      Some(ipc::TauriRenderProcessHandler::new())
    }

    fn browser_process_handler(&self) -> Option<BrowserProcessHandler> {
      Some(browser_client::TauriCefBrowserProcessHandler::new(
        self.context.clone(),
        self.context_initialized.clone(),
        self.deep_link_schemes.clone(),
      ))
    }

    fn on_before_command_line_processing(
      &self,
      process_type: Option<&CefString>,
      command_line: Option<&mut CommandLine>,
    ) {
      let Some(command_line) = command_line else {
        return;
      };

      append_command_line_args(command_line, &self.internal_command_line_args);

      // The browser process is the one launched without a `--type` switch, so CEF hands
      // us an empty (or absent) process type for it.
      let is_browser_process = process_type.is_none_or(|ty| ty.to_string().is_empty());
      if is_browser_process {
        // A second launch of an already-running application is a browser process too,
        // so `Settings::command_line_args_disabled` clears its command line in
        // `BasicStartupComplete` - before Chromium's process singleton relays it to the
        // first instance. `on_already_running_app_relaunch` then receives a command line
        // with no arguments at all and the `myapp://...` URL is lost, which is why the
        // deep link has to be put back here, after CEF's clear and before the singleton.
        //
        // Only deep links are restored; every other argument stays dropped, which is
        // the entire point of the lockdown.
        if self.restore_deep_link_arguments {
          for deep_link in deep_link_arguments(std::env::args().skip(1), &self.deep_link_schemes) {
            command_line.append_argument(Some(&CefString::from(deep_link.as_str())));
          }
        }

        append_command_line_args(command_line, &self.browser_command_line_args);
      }
    }
  }
}

pub fn run_cef_helper_process() {
  let args = cef::args::Args::new();

  // A helper the browser process launched with `--no-sandbox` must not enter the sandbox
  // here: the browser dropped it deliberately - `SandboxPolicy::Disabled`, or a framework
  // outside the app bundle that the sandbox cannot reach - and entering it anyway would
  // only make the library load below fail.
  #[cfg(target_os = "macos")]
  let _sandbox = (!crate::sandbox::launched_without_sandbox()).then(|| {
    let mut sandbox = cef::sandbox::Sandbox::new();
    sandbox.initialize(args.as_main_args());
    sandbox
  });

  #[cfg(target_os = "macos")]
  let _loader = {
    let loader = cef::library_loader::LibraryLoader::new(&std::env::current_exe().unwrap(), true);
    assert!(loader.load());
    loader
  };

  let _ = cef::api_hash(sys::CEF_API_VERSION_LAST, 0);
  let mut app = TauriCefHelperApp::new();
  let _ = cef::execute_process(
    Some(args.as_main_args()),
    Some(&mut app),
    std::ptr::null_mut(),
  );
}

wrap_app! {
  struct TauriCefHelperApp;

  impl App {
    fn render_process_handler(&self) -> Option<RenderProcessHandler> {
      Some(ipc::TauriRenderProcessHandler::new())
    }
  }
}

#[derive(Debug, Clone)]
pub struct CefRuntimeHandle<T: UserEvent> {
  context: RuntimeContext<T>,
}

impl<T: UserEvent> RuntimeHandle<T> for CefRuntimeHandle<T> {
  type Runtime = CefRuntime<T>;

  fn create_proxy(&self) -> <Self::Runtime as Runtime<T>>::EventLoopProxy {
    EventProxy {
      context: self.context.clone(),
    }
  }

  #[cfg(target_os = "macos")]
  fn set_activation_policy(
    &self,
    activation_policy: tauri_runtime::ActivationPolicy,
  ) -> Result<()> {
    let message = Message::EventLoop(EventLoopMessage::SetActivationPolicy(activation_policy));
    self.context.send_message(message)
  }

  #[cfg(target_os = "macos")]
  fn set_dock_visibility(&self, visible: bool) -> Result<()> {
    let message = Message::EventLoop(EventLoopMessage::SetDockVisibility(visible));
    self.context.send_message(message)
  }

  fn request_exit(&self, code: i32) -> Result<()> {
    self.context.send_message(Message::RequestExit(code))
  }

  /// Returns the URL for a custom scheme.
  ///
  /// CEF always uses `http://<scheme>.localhost` or `https://<scheme>.localhost`.
  fn custom_scheme_url(&self, scheme: &str, https: bool) -> String {
    format!(
      "{}://{scheme}.localhost",
      if https { "https" } else { "http" }
    )
  }

  fn webview_version(&self) -> Result<String> {
    crate::webview_version()
  }

  fn create_window<F: Fn(RawWindow<'_>) + Send + 'static>(
    &self,
    pending: PendingWindow<T, Self::Runtime>,
    after_window_creation: Option<F>,
  ) -> Result<DetachedWindow<T, Self::Runtime>> {
    create_window_detached(&self.context, pending, after_window_creation)
  }

  fn create_webview(
    &self,
    window_id: WindowId,
    pending: PendingWebview<T, Self::Runtime>,
  ) -> Result<DetachedWebview<T, Self::Runtime>> {
    create_webview_detached(&self.context, window_id, pending)
  }

  fn run_on_main_thread<F: FnOnce() + Send + 'static>(&self, f: F) -> Result<()> {
    self.context.run_on_main_thread(f)
  }

  fn display_handle(
    &self,
  ) -> std::result::Result<DisplayHandle<'_>, raw_window_handle::HandleError> {
    let raw = event_loop_getter!(self, DisplayHandle)
      .map_err(|_| raw_window_handle::HandleError::Unavailable)??;
    // SAFETY: the descriptor was produced by the live event loop on its own
    // thread; the borrowed handle is valid for as long as the runtime is.
    Ok(unsafe { DisplayHandle::borrow_raw(raw.0) })
  }

  fn primary_monitor(&self) -> Result<Option<Monitor>> {
    event_loop_getter!(self, PrimaryMonitor)?
  }

  fn monitor_from_point(&self, x: f64, y: f64) -> Result<Option<Monitor>> {
    let (tx, rx) = mpsc::channel();
    self
      .context
      .send_message(Message::EventLoop(EventLoopMessage::MonitorFromPoint(
        tx, x, y,
      )))?;
    rx.recv().map_err(|_| Error::FailedToReceiveMessage)?
  }

  fn available_monitors(&self) -> Result<Vec<Monitor>> {
    event_loop_getter!(self, AvailableMonitors)?
  }

  fn cursor_position(&self) -> Result<PhysicalPosition<f64>> {
    event_loop_getter!(self, CursorPosition)?
  }

  fn set_theme(&self, theme: Option<Theme>) {
    let message = Message::EventLoop(EventLoopMessage::SetTheme(theme));
    let _ = self.context.send_message(message);
  }

  #[cfg(target_os = "macos")]
  fn show(&self) -> Result<()> {
    let message = Message::EventLoop(EventLoopMessage::ShowApplication);
    self.context.send_message(message)
  }

  #[cfg(target_os = "macos")]
  fn hide(&self) -> Result<()> {
    let message = Message::EventLoop(EventLoopMessage::HideApplication);
    self.context.send_message(message)
  }

  fn set_device_event_filter(&self, filter: DeviceEventFilter) {
    let message = Message::EventLoop(EventLoopMessage::SetDeviceEventFilter(filter));
    let _ = self.context.send_message(message);
  }

  #[cfg(any(target_os = "macos", target_os = "ios"))]
  fn fetch_data_store_identifiers<F: FnOnce(Vec<[u8; 16]>) + Send + 'static>(
    &self,
    cb: F,
  ) -> Result<()> {
    cb(Vec::new());
    Ok(())
  }

  #[cfg(any(target_os = "macos", target_os = "ios"))]
  fn remove_data_store<F: FnOnce(Result<()>) + Send + 'static>(
    &self,
    _uuid: [u8; 16],
    cb: F,
  ) -> Result<()> {
    cb(Ok(()));
    Ok(())
  }
}

pub struct CefRuntime<T: UserEvent = tauri::EventLoopMessage> {
  event_loop: EventLoop,
  receiver: Receiver<Message<T>>,
  context: RuntimeContext<T>,
  scheme_registry: request_handler::SchemeRegistry,
  #[cfg(target_os = "macos")]
  _app_delegate: Option<objc2::rc::Retained<crate::platform::macos::AppDelegate>>,
}

impl<T: UserEvent> fmt::Debug for CefRuntime<T> {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("CefRuntime").finish()
  }
}

/// CEF installs Chromium's shutdown signal handlers for SIGINT/SIGTERM/SIGHUP
/// inside `cef::initialize`. Those handlers hand the signal to Chrome's own exit
/// machinery, which never quits the winit loop this runtime owns, so the process
/// just ignores the signal. Because the handler disarms itself on delivery, it
/// takes a second Ctrl+C to kill the app by default disposition.
///
/// Put the process's original signal policy back so termination signals behave
/// the way they do for any other app (and the way they already do under
/// `tauri-runtime-wry`, which installs no handlers at all).
#[cfg(any(
  target_os = "linux",
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "openbsd",
  target_os = "netbsd"
))]
struct TerminationSignals {
  sigint: Option<libc::sigaction>,
  sigterm: Option<libc::sigaction>,
  sighup: Option<libc::sigaction>,
}

#[cfg(any(
  target_os = "linux",
  target_os = "dragonfly",
  target_os = "freebsd",
  target_os = "openbsd",
  target_os = "netbsd"
))]
impl TerminationSignals {
  fn capture() -> Self {
    Self {
      sigint: Self::capture_one(libc::SIGINT),
      sigterm: Self::capture_one(libc::SIGTERM),
      sighup: Self::capture_one(libc::SIGHUP),
    }
  }

  /// Must run *after* `cef::initialize`, which is the only place CEF installs
  /// these handlers; nothing reinstalls them later, so one restore is enough.
  fn restore(&self) {
    Self::restore_one(libc::SIGINT, self.sigint);
    Self::restore_one(libc::SIGTERM, self.sigterm);
    Self::restore_one(libc::SIGHUP, self.sighup);
  }

  fn capture_one(sig: libc::c_int) -> Option<libc::sigaction> {
    let mut action = std::mem::MaybeUninit::<libc::sigaction>::uninit();
    if unsafe { libc::sigaction(sig, std::ptr::null(), action.as_mut_ptr()) } == 0 {
      Some(unsafe { action.assume_init() })
    } else {
      None
    }
  }

  fn restore_one(sig: libc::c_int, previous: Option<libc::sigaction>) {
    let Some(previous) = previous else {
      return;
    };

    unsafe { libc::sigaction(sig, &previous, std::ptr::null_mut()) };
  }
}

impl<T: UserEvent> CefRuntime<T> {
  fn init(
    mut event_loop_builder: EventLoopBuilder,
    runtime_args: RuntimeInitArgs<Cef>,
  ) -> Result<Self> {
    // Snapshot before CEF can touch anything, so we can tell an embedder's own
    // signal policy apart from the handlers CEF installs in `cef::initialize`.
    #[cfg(any(
      target_os = "linux",
      target_os = "dragonfly",
      target_os = "freebsd",
      target_os = "openbsd",
      target_os = "netbsd"
    ))]
    let pre_cef_signals = TerminationSignals::capture();

    let args = cef::args::Args::new();

    #[cfg(target_os = "macos")]
    let is_helper = is_cef_helper_process();

    #[cfg(target_os = "macos")]
    let (_sandbox, _loader) = {
      // As in `run_cef_helper_process`: only a helper enters the sandbox, and only when
      // the browser process that launched it did not already drop the sandbox.
      let sandbox = if is_helper && !crate::sandbox::launched_without_sandbox() {
        let mut sandbox = cef::sandbox::Sandbox::new();
        sandbox.initialize(args.as_main_args());
        Some(sandbox)
      } else {
        None
      };

      let loader =
        cef::library_loader::LibraryLoader::new(&std::env::current_exe().unwrap(), is_helper);
      assert!(loader.load());

      (sandbox, loader)
    };

    #[cfg(target_os = "macos")]
    if !is_helper {
      crate::platform::macos::setup_application();
    }

    // The CEF API version table must be initialized before any other CEF call
    // (e.g. `args.as_cmd_line()` below), otherwise the process crashes with no
    // diagnostics.
    let version = runtime_args
      .runtime_init_attrs
      .api_version
      .unwrap_or(sys::CEF_API_VERSION_LAST);
    let _ = cef::api_hash(version, 0);

    // Handle CEF subprocesses (renderer/GPU/utility) before any browser-only
    // setup such as building the event loop, creating cache directories, or the
    // runtime context. The browser (main) process has no `type` switch;
    // subprocesses are launched with one (e.g. `--type=renderer`).
    let is_browser_process = args
      .as_cmd_line()
      .map(|cmd| cmd.has_switch(Some(&CefString::from("type"))) != 1)
      .unwrap_or(true);

    if !is_browser_process {
      let mut helper_app = TauriCefHelperApp::new();
      let ret = cef::execute_process(
        Some(args.as_main_args()),
        Some(&mut helper_app),
        std::ptr::null_mut(),
      );
      // A subprocess finished its work; exit with its exit code instead of
      // falling through to browser runtime initialization.
      std::process::exit(ret.max(0));
    }

    let Cef {
      command_line_args,
      deep_link_schemes,
      cache_path: cache_path_override,
      secret_storage,
      profile_preferences,
      allow_chromium_command_line_args,
      log_file,
      log_severity,
      locale,
      accept_language_list,
      sandbox: sandbox_policy,
      settings_callback,
      // Already applied, above, before the first CEF call.
      api_version: _,
    } = runtime_args.runtime_init_attrs;

    // Switches every process gets, and switches only the browser process gets. See the
    // `TauriCefApp` fields for why the split exists.
    #[allow(unused_mut)]
    let mut internal_command_line_args: Vec<(String, Option<String>)> = Vec::new();
    #[allow(unused_mut)]
    let mut browser_command_line_args: Vec<(String, Option<String>)> = Vec::new();

    // `os_crypt` only ever runs in the browser process, so these are browser-only
    // switches. See `SecretStorage` for what each one costs.
    #[cfg(target_os = "macos")]
    {
      let mock_keychain = match secret_storage {
        SecretStorage::Auto => tauri::is_dev(),
        SecretStorage::Mock => true,
        SecretStorage::System => false,
      };
      if mock_keychain {
        browser_command_line_args.push(("--use-mock-keychain".to_string(), None));
      }
    }
    #[cfg(any(
      target_os = "linux",
      target_os = "dragonfly",
      target_os = "freebsd",
      target_os = "netbsd",
      target_os = "openbsd"
    ))]
    {
      // `basic` skips the D-Bus secret portal, libsecret and KWallet key providers, any
      // of which can block startup on a keyring-unlock dialog or fail outright in a
      // headless session. That is a development annoyance worth avoiding and a shipped
      // application's cookie encryption worth keeping, so `Auto` splits the same way it
      // does for the macOS keychain.
      let basic_password_store = match secret_storage {
        SecretStorage::Auto => tauri::is_dev(),
        SecretStorage::Mock => true,
        SecretStorage::System => false,
      };
      if basic_password_store {
        browser_command_line_args.push(("--password-store".to_string(), Some("basic".to_string())));
      }
    }

    // One decision on every platform, so a lost sandbox is always something the policy
    // asked for and is always logged. This used to be `!cfg!(feature = "sandbox")` off
    // Linux, which handed a consumer building with `default-features = false` - as the
    // workspace root does - a silently unsandboxed Chromium on Windows and macOS.
    //
    // On Linux and the BSDs the policy is also weighed against the system, because
    // Chromium aborts with "No usable sandbox!" when its zygote host finds neither usable
    // unprivileged user namespaces nor the setuid `chrome-sandbox` helper, so an AppImage
    // on a system that restricts namespaces cannot start at all.
    let no_sandbox = {
      let decision = crate::sandbox::resolve_sandbox_decision(sandbox_policy);
      if let crate::sandbox::SandboxDecision::Disable(reason) = decision {
        log::warn!(
          "running Chromium without a sandbox: {}. A compromised renderer process runs with the full privileges of the current user.",
          reason.message()
        );
      }
      matches!(decision, crate::sandbox::SandboxDecision::Disable(_))
    };
    // Windows encrypts with DPAPI, which needs no switch and prompts for nothing.
    #[cfg(windows)]
    let _ = secret_storage;

    let cache_path = cache_path_override.unwrap_or_else(|| {
      let cache_base = dirs::cache_dir().unwrap_or_else(std::env::temp_dir);
      cache_base.join(&runtime_args.identifier).join("cef")
    });
    let _ = create_dir_all(&cache_path);

    // Force X11 usage on Linux.
    //
    // Put in the list that is not conditioned on the process type: we have not verified
    // that Chromium propagates `ozone-platform` to the GPU process, and getting it wrong
    // there breaks rendering on Linux outright.
    #[cfg(any(
      target_os = "linux",
      target_os = "dragonfly",
      target_os = "freebsd",
      target_os = "netbsd",
      target_os = "openbsd"
    ))]
    {
      internal_command_line_args.push(("--ozone-platform".to_string(), Some("x11".to_string())));
      event_loop_builder.with_x11();
    }

    #[cfg(windows)]
    if let Some(hook) = runtime_args.msg_hook {
      use winit::platform::windows::EventLoopBuilderExtWindows;
      event_loop_builder.with_msg_hook(hook);
    }

    #[cfg(target_os = "macos")]
    event_loop_builder.with_default_menu(false);

    let event_loop = event_loop_builder
      .build()
      .map_err(|_| Error::CreateWindow)?;
    let proxy = event_loop.create_proxy();
    let (sender, receiver) = mpsc::channel();
    let context_initialized = Arc::new(AtomicBool::new(false));
    let cef_pump = CefExternalPump::new();
    let context = RuntimeContext {
      sender: sender.clone(),
      proxy: proxy.clone(),
      main_thread_id: std::thread::current().id(),
      next_window_id: Default::default(),
      next_webview_id: Default::default(),
      next_window_event_id: Default::default(),
      next_webview_event_id: Default::default(),
      current_dispatch: Default::default(),
      app_wide_theme: Default::default(),
      cef_pump,
      cache_path: Arc::new(cache_path.clone()),
      profile_preferences: Arc::new(profile_preferences),
    };

    internal_command_line_args.push(("--no-first-run".to_string(), None));

    // Appended last so an application switch overrides a runtime default with the same
    // name: Chromium's command line keeps the last value appended for a given switch.
    browser_command_line_args.extend(command_line_args);

    // Shipped applications ignore Chromium switches passed on their own command line:
    // otherwise anyone able to launch the app can also launch it with
    // `--remote-debugging-port` and drive it over the DevTools protocol, or with
    // `--disable-web-security`, `--proxy-server`, `--host-resolver-rules` or
    // `--ssl-key-log-file`, all of which Chromium honours. CEF clears the command line
    // before applying `Settings` and before calling `on_before_command_line_processing`,
    // so the switches this runtime and the application configure still take effect.
    //
    // Tauri's own CLI parsing and its cold-start deep link handling read
    // `std::env::args()`, which Chromium never touches, so they keep working. The one
    // thing the clear does break is the *relaunch* deep link path, where Chromium's
    // process singleton relays this process's command line to the already-running
    // instance; `TauriCefApp::on_before_command_line_processing` puts the deep link back
    // for that reason.
    let command_line_args_disabled = !(allow_chromium_command_line_args || tauri::is_dev());

    let mut app = TauriCefApp::build(TauriCefAppArgs {
      context: context.clone(),
      context_initialized: context_initialized.clone(),
      deep_link_schemes,
      restore_deep_link_arguments: command_line_args_disabled,
      internal_command_line_args,
      browser_command_line_args,
    });

    // Subprocesses already exited above, so this must be the browser process;
    // `execute_process` returns -1 there to signal normal startup should follow.
    let ret = cef::execute_process(
      Some(args.as_main_args()),
      Some(&mut app),
      std::ptr::null_mut(),
    );
    assert_eq!(
      ret, -1,
      "CEF browser process unexpectedly returned from execute_process"
    );

    // Chromium drops a `debug.log` into the *process working directory* when no log file
    // is configured, which for an installed application is wherever the user launched it
    // from. Keep it next to the rest of the runtime's state instead.
    let log_file = log_file.unwrap_or_else(|| cache_path.join("cef.log"));
    // CEF logs at INFO by default, which grows that file quickly in a long-running app.
    let log_severity = log_severity.unwrap_or(if tauri::is_dev() {
      LogSeverity::DEFAULT
    } else {
      LogSeverity::WARNING
    });

    let mut settings = cef::Settings {
      // Only this, never a `--no-sandbox` push of our own: CEF appends that switch itself
      // from the setting, and does it before `on_before_command_line_processing` runs.
      // One mechanism means the setting and the switch cannot end up disagreeing.
      no_sandbox: no_sandbox as std::os::raw::c_int,
      cache_path: cache_path.to_string_lossy().to_string().as_str().into(),
      command_line_args_disabled: command_line_args_disabled as std::os::raw::c_int,
      log_file: log_file.to_string_lossy().to_string().as_str().into(),
      log_severity,
      external_message_pump: 1,
      ..Default::default()
    };

    // Left at CEF's defaults unless the application asked for something else: the
    // bundler only ships the `en-US` locale pak, so a locale we picked on our own would
    // leave Chromium without its localized resources.
    if let Some(locale) = locale {
      settings.locale = locale.as_str().into();
    }
    if let Some(accept_language_list) = accept_language_list {
      settings.accept_language_list = accept_language_list.as_str().into();
    }

    if let Some(callback) = settings_callback {
      callback(&mut settings);
    }
    if cef::initialize(
      Some(args.as_main_args()),
      Some(&settings),
      Some(&mut app),
      std::ptr::null_mut(),
    ) != 1
    {
      return Err(Error::WebviewRuntimeNotInstalled);
    }

    #[cfg(any(
      target_os = "linux",
      target_os = "dragonfly",
      target_os = "freebsd",
      target_os = "openbsd",
      target_os = "netbsd"
    ))]
    pre_cef_signals.restore();

    #[cfg(target_os = "macos")]
    let app_delegate = if !is_helper {
      use crate::platform::macos::AppDelegateEvent;

      let context_ = context.clone();
      let handler = Box::new(move |event| match event {
        AppDelegateEvent::TryTerminate => {
          let _ = context_.send_message(Message::RequestExit(0));
        }
        AppDelegateEvent::Reopen {
          has_visible_windows,
        } => {
          let _ = context_.send_message(Message::Reopen {
            has_visible_windows,
          });
        }
        AppDelegateEvent::AccessibilityChanged { enabled } => {
          let _ = context_.send_message(Message::AccessibilityChanged { enabled });
        }
        AppDelegateEvent::OpenURLs { urls } => {
          let _ = context_.send_message(Message::Opened(urls));
        }
      });
      let app_delegate = crate::platform::macos::set_application_event_handler(handler);
      Some(app_delegate)
    } else {
      None
    };

    // Wait for the CEF context to initialize before returning, so that the runtime is ready to create browsers.
    while !context_initialized.load(Ordering::SeqCst) {
      context.cef_pump.do_work();
      std::thread::sleep(Duration::from_millis(1));
    }

    Ok(Self {
      event_loop,
      receiver,
      context,
      scheme_registry: Default::default(),
      #[cfg(target_os = "macos")]
      _app_delegate: app_delegate,
    })
  }
}

impl<T: UserEvent> Runtime<T> for CefRuntime<T> {
  type WindowDispatcher = CefWindowDispatcher<T>;
  type WebviewDispatcher = CefWebviewDispatcher<T>;
  type Handle = CefRuntimeHandle<T>;
  type EventLoopProxy = EventProxy<T>;
  type RuntimeWebviewAttributes = CefWebviewAttributes;
  type Webview = Webview;
  type RuntimeInitAttrs = Cef;
  type WindowOpener = NewWindowOpener;

  fn new(args: RuntimeInitArgs<Self::RuntimeInitAttrs>) -> Result<Self> {
    Self::init(EventLoopBuilder::default(), args)
  }

  #[cfg(any(
    windows,
    target_os = "linux",
    target_os = "dragonfly",
    target_os = "freebsd",
    target_os = "netbsd",
    target_os = "openbsd"
  ))]
  fn new_any_thread(args: RuntimeInitArgs<Self::RuntimeInitAttrs>) -> Result<Self> {
    let mut event_loop_builder = EventLoopBuilder::default();
    event_loop_builder.with_any_thread(true);
    Self::init(event_loop_builder, args)
  }

  fn create_proxy(&self) -> Self::EventLoopProxy {
    EventProxy {
      context: self.context.clone(),
    }
  }

  fn handle(&self) -> Self::Handle {
    CefRuntimeHandle {
      context: self.context.clone(),
    }
  }

  fn create_window<F: Fn(RawWindow<'_>) + Send + 'static>(
    &self,
    pending: PendingWindow<T, Self>,
    after_window_creation: Option<F>,
  ) -> Result<DetachedWindow<T, Self>> {
    create_window_detached(&self.context, pending, after_window_creation)
  }

  fn create_webview(
    &self,
    window_id: WindowId,
    pending: PendingWebview<T, Self>,
  ) -> Result<DetachedWebview<T, Self>> {
    create_webview_detached(&self.context, window_id, pending)
  }

  fn primary_monitor(&self) -> Option<Monitor> {
    event_loop_getter!(self, PrimaryMonitor)
      .flatten()
      .ok()
      .unwrap_or_default()
  }

  fn monitor_from_point(&self, x: f64, y: f64) -> Option<Monitor> {
    let (tx, rx) = mpsc::channel();
    self
      .context
      .send_message(Message::EventLoop(EventLoopMessage::MonitorFromPoint(
        tx, x, y,
      )))
      .and_then(|_| rx.recv().map_err(|_| Error::FailedToReceiveMessage))
      .ok()?
      .ok()
      .unwrap_or_default()
  }

  fn available_monitors(&self) -> Vec<Monitor> {
    event_loop_getter!(self, AvailableMonitors)
      .flatten()
      .ok()
      .unwrap_or_default()
  }

  fn cursor_position(&self) -> Result<PhysicalPosition<f64>> {
    event_loop_getter!(self, CursorPosition)?
  }

  fn set_theme(&self, theme: Option<Theme>) {
    let message = Message::EventLoop(EventLoopMessage::SetTheme(theme));
    let _ = self.context.send_message(message);
  }

  #[cfg(target_os = "macos")]
  fn set_activation_policy(&mut self, activation_policy: tauri_runtime::ActivationPolicy) {
    let message = Message::EventLoop(EventLoopMessage::SetActivationPolicy(activation_policy));
    let _ = self.context.send_message(message);
  }

  #[cfg(target_os = "macos")]
  fn set_dock_visibility(&mut self, visible: bool) {
    let message = Message::EventLoop(EventLoopMessage::SetDockVisibility(visible));
    let _ = self.context.send_message(message);
  }

  #[cfg(target_os = "macos")]
  fn show(&self) {
    let message = Message::EventLoop(EventLoopMessage::ShowApplication);
    let _ = self.context.send_message(message);
  }

  #[cfg(target_os = "macos")]
  fn hide(&self) {
    let message = Message::EventLoop(EventLoopMessage::HideApplication);
    let _ = self.context.send_message(message);
  }

  fn set_device_event_filter(&mut self, filter: DeviceEventFilter) {
    self
      .event_loop
      .listen_device_events(device_event_filter_to_winit(filter));
  }

  fn run_iteration<F: FnMut(RunEvent<T>) + 'static>(&mut self, mut callback: F) {
    while let Ok(message) = self.receiver.try_recv() {
      if let Message::UserEvent(event) = message {
        callback(RunEvent::UserEvent(event));
      }
    }
    self.context.cef_pump.do_work();
    callback(RunEvent::MainEventsCleared);
  }

  fn run_return<F: FnMut(RunEvent<T>) + 'static>(self, callback: F) -> i32 {
    self.run(callback);
    // TODO: return the exit code from the runtime, if possible. For now, always return 0
    0
  }

  fn run<F: FnMut(RunEvent<T>) + 'static>(self, callback: F) {
    let app = WinitCefApp::new(
      self.context,
      self.receiver,
      Box::new(callback),
      self.scheme_registry,
    );
    let _ = self.event_loop.run_app(app);
    cef::shutdown();
  }
}

#[cfg(test)]
mod deep_link_argument_tests {
  use super::deep_link_arguments;

  fn schemes() -> Vec<String> {
    vec!["myapp".to_string(), "my-other-app".to_string()]
  }

  fn filter(args: &[&str]) -> Vec<String> {
    deep_link_arguments(args.iter().map(|arg| (*arg).to_string()), &schemes())
  }

  #[test]
  fn keeps_configured_deep_links_in_order() {
    assert_eq!(
      filter(&["myapp://open/one", "my-other-app://open/two"]),
      vec![
        "myapp://open/one".to_string(),
        "my-other-app://open/two".to_string(),
      ]
    );
  }

  #[test]
  fn drops_everything_that_is_not_a_configured_deep_link() {
    // The lockdown exists so that none of these reach Chromium's command line, and a
    // URL with an unconfigured scheme is not this application's deep link either.
    assert!(
      filter(&[
        "--remote-debugging-port=9222",
        "--disable-web-security",
        "/home/user/document.txt",
        "not a url",
        "",
        "https://example.com",
        "otherapp://open",
      ])
      .is_empty()
    );
  }

  #[test]
  fn an_empty_scheme_list_keeps_nothing() {
    assert!(deep_link_arguments(["myapp://open".to_string()], &[]).is_empty());
  }

  #[test]
  fn scheme_matching_is_exact() {
    // `on_already_running_app_relaunch` compares schemes the same way, so anything
    // matched loosely here would be re-appended and then ignored on the other end.
    // `Url::parse` already lowercases the scheme it reports, which is why the
    // upper-case spelling below still matches.
    assert_eq!(filter(&["MYAPP://open"]), vec!["MYAPP://open".to_string()]);
    assert!(filter(&["myapp2://open", "myap://open"]).is_empty());
  }
}
