// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Everything this example configures on the CEF runtime itself, in one place.
//!
//! The runtime is selected and configured before the application is built, by
//! passing [`Cef`] to `tauri::Builder::runtime`. Most of what is here has no
//! equivalent on the system webview runtime: Chromium's process sandbox, the key
//! its cookie jar is encrypted with, the profile preferences of the Chrome
//! features an app window inherits, and its command line.
//!
//! Every knob is driven by an environment variable so the effect of each one can
//! be seen without editing this file. See the example's `README.md`.

use std::path::PathBuf;

use serde::Serialize;
use tauri_runtime_cef::{
  CEF_API_VERSION_LAST, Cef, SandboxPolicy, SecretStorage, cef::LogSeverity,
};

/// Kept in sync with the `identifier` of `tauri.conf.json`.
const IDENTIFIER: &str = "com.tauri.cef-example";

/// The deep link scheme the runtime watches for on a relaunch.
///
/// Schemes declared under `plugins > deep-link > desktop` in `tauri.conf.json`
/// are added to this list automatically by [`Cef::apply_config`], so an app
/// using the deep link plugin does not have to name them twice.
pub const DEEP_LINK_SCHEME: &str = "tauri-cef-example";

/// The CEF configuration of this run, kept around so the frontend can show it.
#[derive(Clone)]
pub struct RuntimeConfig {
  sandbox: SandboxPolicy,
  secret_storage: SecretStorage,
  safe_browsing: bool,
  password_manager: bool,
  log_severity: Option<(String, LogSeverity)>,
  allow_chromium_command_line_args: bool,
  autoplay_without_gesture: bool,
  cache_path: PathBuf,
  log_file: PathBuf,
  cef_api_version: i32,
}

impl RuntimeConfig {
  pub fn from_env() -> Self {
    // The runtime defaults to `{user cache}/{identifier}/cef`. This example keeps
    // its own directory so trying `SecretStorage` variants here cannot invalidate
    // the cookie jar of another app built from this repository: cookies encrypted
    // under one key are unreadable under another.
    let cache_path = dirs::cache_dir()
      .unwrap_or_else(std::env::temp_dir)
      .join(IDENTIFIER)
      .join("cef");

    Self {
      sandbox: match env("CEF_EXAMPLE_SANDBOX").as_deref() {
        // Never run unsandboxed, even when that means Chromium aborts at startup.
        Some("required") => SandboxPolicy::Required,
        // Every renderer then runs with the full privileges of the user.
        Some("disabled") => SandboxPolicy::Disabled,
        // Keeps the sandbox everywhere except an AppImage on a system that offers
        // no way to sandbox at all, where Chromium would otherwise abort.
        _ => SandboxPolicy::Auto,
      },
      secret_storage: match env("CEF_EXAMPLE_SECRET_STORAGE").as_deref() {
        // Chromium's hard-coded key: no protection at rest, but it runs where
        // there is no keyring at all (a container, a headless session, CI).
        Some("mock") => SecretStorage::Mock,
        // The OS secret store in every build profile, keyring prompt included.
        Some("system") => SecretStorage::System,
        _ => SecretStorage::Auto,
      },
      // Left on by the runtime. An app whose webview only ever loads its own
      // content can switch Safe Browsing off and save the network traffic.
      safe_browsing: !env_is(&env("CEF_EXAMPLE_SAFE_BROWSING"), "off"),
      // Disabled by the runtime, because the "Save password?" bubble draws over
      // an app's own UI. A browser-shaped app can ask for it back.
      password_manager: env_is(&env("CEF_EXAMPLE_PASSWORD_MANAGER"), "on"),
      log_severity: match env("CEF_EXAMPLE_LOG_SEVERITY").as_deref() {
        Some(name @ "verbose") => Some((name.into(), LogSeverity::VERBOSE)),
        Some(name @ "info") => Some((name.into(), LogSeverity::INFO)),
        Some(name @ "warning") => Some((name.into(), LogSeverity::WARNING)),
        Some(name @ "error") => Some((name.into(), LogSeverity::ERROR)),
        // Not "no logging": CEF maps it to a FATAL-only minimum level, and FATAL
        // messages still reach stderr.
        Some(name @ "disable") => Some((name.into(), LogSeverity::DISABLE)),
        // The runtime's own default: INFO in development, WARNING in release.
        _ => None,
      },
      // Release builds ignore Chromium switches passed on their own command line,
      // so a shipped app cannot be relaunched with `--remote-debugging-port` or
      // `--disable-web-security` by whoever can start it. Development builds
      // always keep the command line enabled, whatever this says.
      allow_chromium_command_line_args: env_is(&env("CEF_EXAMPLE_CHROMIUM_ARGS"), "on"),
      // Drives a raw Chromium switch rather than a `Cef` method, so the page can
      // be compared with Chromium's own default of requiring a user gesture.
      autoplay_without_gesture: !env_is(&env("CEF_EXAMPLE_AUTOPLAY"), "off"),
      // The version of the CEF API this build was compiled against, which is
      // also `Cef`'s own default. Both sides of a CEF application — the browser
      // process and every helper process the entry point runs — have to agree on
      // it, and they do here because they are the same executable.
      cef_api_version: CEF_API_VERSION_LAST,
      log_file: cache_path.join("cef.log"),
      cache_path,
    }
  }

  /// The CEF API version the runtime was told to declare.
  pub fn cef_api_version(&self) -> i32 {
    self.cef_api_version
  }

  /// The runtime attributes handed to `tauri::Builder::runtime`.
  pub fn cef(&self) -> Cef {
    let cef = Cef::default()
      .sandbox(self.sandbox)
      .secret_storage(self.secret_storage)
      // Already the default; named here because the browser process and the
      // helper processes must declare the same one.
      .cef_api_version(self.cef_api_version)
      .root_cache_path(&self.cache_path)
      // Without this CEF drops a `debug.log` next to the executable, which for an
      // installed app is a directory that is often not even writable.
      .log_file(&self.log_file)
      .allow_chromium_command_line_args(self.allow_chromium_command_line_args)
      // Boolean Chromium profile preferences, applied to every webview's request
      // context after the runtime's own defaults, so they can turn a preference
      // the runtime disabled back on as well as turn something else off.
      .profile_preference("safebrowsing.enabled", self.safe_browsing)
      .profile_preference("credentials_enable_service", self.password_manager)
      // Sent as the `Accept-Language` header and reported by `navigator.languages`,
      // which the frontend reads back to prove the setting arrived.
      .accept_language_list(ACCEPT_LANGUAGE_LIST)
      // The plural form of the raw switch set further down. The browser process
      // is the only command line an application ever sets — CEF warns that
      // writing a helper process's own may crash it — and Chromium forwards to
      // each child the switches it needs. `js-flags` is one of those: it is set
      // here, on the browser process, and ends up in the renderer, where the
      // frontend finds the `window.gc` that `--expose-gc` adds.
      .command_line_args([
        ("js-flags", Some("--expose-gc")),
        ("no-default-browser-check", None),
      ])
      // Picked up by the runtime's relaunch hook, so `tauri-cef-example://...`
      // reaches this process when the app is already running.
      .deep_link_schemes([DEEP_LINK_SCHEME])
      // The escape hatch for anything `Cef` has no method for: the `cef::Settings`
      // struct as CEF receives it. Session cookies normally die with the process.
      .with_settings(|settings| settings.persist_session_cookies = 1);

    // A raw Chromium switch, applied to the browser process only, and the one
    // knob here that is observable as behaviour rather than as a reported value:
    // an `AudioContext` created with no user gesture behind it starts `running`
    // with this switch and `suspended` without it.
    let cef = if self.autoplay_without_gesture {
      cef.command_line_arg("autoplay-policy", Some("no-user-gesture-required"))
    } else {
      cef
    };

    // `Cef::locale` is deliberately not set: the bundler packages only the `en-US`
    // locale pak, so naming another locale leaves Chromium unable to load the
    // strings of its own UI (context menus, error pages, form controls).

    match self.log_severity {
      Some((_, severity)) => cef.log_severity(severity),
      None => cef,
    }
  }

  /// What the frontend displays in its "runtime configuration" panel.
  pub fn describe(&self) -> Vec<ConfiguredValue> {
    vec![
      ConfiguredValue::new(
        "sandbox",
        format!("{:?}", self.sandbox),
        "CEF_EXAMPLE_SANDBOX=auto|required|disabled",
      ),
      ConfiguredValue::new(
        "secret_storage",
        format!("{:?}", self.secret_storage),
        "CEF_EXAMPLE_SECRET_STORAGE=auto|mock|system",
      ),
      ConfiguredValue::new(
        "profile_preference(\"safebrowsing.enabled\")",
        self.safe_browsing.to_string(),
        "CEF_EXAMPLE_SAFE_BROWSING=on|off",
      ),
      ConfiguredValue::new(
        "profile_preference(\"credentials_enable_service\")",
        self.password_manager.to_string(),
        "CEF_EXAMPLE_PASSWORD_MANAGER=on|off",
      ),
      ConfiguredValue::new(
        "log_severity",
        match &self.log_severity {
          Some((name, _)) => name.clone(),
          None => "default (INFO in dev, WARNING in release)".into(),
        },
        "CEF_EXAMPLE_LOG_SEVERITY=verbose|info|warning|error|disable",
      ),
      ConfiguredValue::new(
        "allow_chromium_command_line_args",
        self.allow_chromium_command_line_args.to_string(),
        "CEF_EXAMPLE_CHROMIUM_ARGS=on|off (development builds always allow them)",
      ),
      ConfiguredValue::new(
        "root_cache_path",
        self.cache_path.display().to_string(),
        "defaults to {user cache}/{identifier}/cef",
      ),
      ConfiguredValue::new(
        "log_file",
        self.log_file.display().to_string(),
        "defaults to cef.log inside the cache directory",
      ),
      ConfiguredValue::new(
        "accept_language_list",
        ACCEPT_LANGUAGE_LIST.to_string(),
        "read back below through navigator.languages",
      ),
      ConfiguredValue::new(
        "command_line_arg(\"autoplay-policy\")",
        if self.autoplay_without_gesture {
          "no-user-gesture-required".into()
        } else {
          "unset, so Chromium's own default applies".to_string()
        },
        "CEF_EXAMPLE_AUTOPLAY=on|off, read back below as an AudioContext's state",
      ),
      ConfiguredValue::new(
        "command_line_args([\"js-flags\", \"no-default-browser-check\"])",
        "--expose-gc".into(),
        "browser-process switches; Chromium forwards js-flags to the renderer",
      ),
      ConfiguredValue::new(
        "cef_api_version",
        self.cef_api_version.to_string(),
        "CEF_API_VERSION_LAST, which both processes of the app must agree on",
      ),
      ConfiguredValue::new(
        "locale",
        "unset".into(),
        "the bundler ships only the en-US pak, so naming another locale breaks Chromium's own UI",
      ),
      ConfiguredValue::new(
        "deep_link_schemes",
        format!("{DEEP_LINK_SCHEME}://"),
        "config schemes of the deep-link plugin are added automatically",
      ),
      ConfiguredValue::new(
        "with_settings",
        "persist_session_cookies = 1".into(),
        "the raw cef::Settings, for what Cef has no method for",
      ),
    ]
  }
}

/// Languages this app asks for, in the `Accept-Language` header syntax.
const ACCEPT_LANGUAGE_LIST: &str = "en-US,en;q=0.9,pt-BR;q=0.8";

/// One configured value, as the frontend renders it.
#[derive(Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConfiguredValue {
  pub name: String,
  pub value: String,
  pub hint: String,
}

impl ConfiguredValue {
  fn new(name: &str, value: String, hint: &str) -> Self {
    Self {
      name: name.into(),
      value,
      hint: hint.into(),
    }
  }
}

fn env(name: &str) -> Option<String> {
  std::env::var(name)
    .ok()
    .map(|value| value.trim().to_lowercase())
}

fn env_is(value: &Option<String>, expected: &str) -> bool {
  value.as_deref() == Some(expected)
}
