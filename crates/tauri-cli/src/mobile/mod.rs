// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  ConfigValue, Result,
  error::{Context, ErrorExt},
  helpers::config::{Config as TauriConfig, ConfigMetadata, reload_config},
  interface::{AppInterface, AppSettings, DevProcess, Options as InterfaceOptions},
};
use heck::ToSnekCase;
use jsonrpsee::core::client::{Client, ClientBuilder, ClientT};
use jsonrpsee::server::{HttpRequest, HttpResponse, RpcModule, ServerBuilder, ServerHandle};
use jsonrpsee::types::ErrorObjectOwned;
use jsonrpsee_client_transport::ws::WsTransportClientBuilder;
use jsonrpsee_core::{BoxError, rpc_params};
use rand::distr::{Alphanumeric, SampleString};
use serde::{Deserialize, Serialize};

use cargo_mobile2::{
  ChildHandle,
  config::app::{App, Raw as RawAppConfig},
  env::Error as EnvError,
  opts::{NoiseLevel, Profile},
};
use std::{
  collections::HashMap,
  env::set_var,
  ffi::OsString,
  fmt::{Display, Write},
  fs::{OpenOptions, create_dir_all, read_to_string, remove_file},
  future::Future,
  io::Write as _,
  net::{AddrParseError, IpAddr, Ipv4Addr, SocketAddr},
  path::{Path, PathBuf},
  pin::Pin,
  process::{ExitStatus, exit},
  str::FromStr,
  sync::{
    Arc, OnceLock,
    atomic::{AtomicBool, Ordering},
  },
  task::Poll,
};
use tokio::runtime::Runtime;

#[cfg(not(windows))]
use cargo_mobile2::env::Env;
#[cfg(windows)]
use cargo_mobile2::os::Env;

pub mod android;
mod init;
#[cfg(target_os = "macos")]
pub mod ios;

const MIN_DEVICE_MATCH_SCORE: isize = 0;

#[derive(Clone)]
pub struct DevChild {
  child: Arc<ChildHandle>,
  manually_killed_process: Arc<AtomicBool>,
}

impl DevChild {
  fn new(handle: ChildHandle) -> Self {
    Self {
      child: Arc::new(handle),
      manually_killed_process: Default::default(),
    }
  }
}

impl DevProcess for DevChild {
  fn kill(&self) -> std::io::Result<()> {
    self.child.kill()?;
    self.manually_killed_process.store(true, Ordering::SeqCst);
    Ok(())
  }

  fn wait(&self) -> std::io::Result<ExitStatus> {
    self.child.wait().map(|o| o.status)
  }

  fn manually_killed_process(&self) -> bool {
    self.manually_killed_process.load(Ordering::SeqCst)
  }
}

#[derive(PartialEq, Eq, Copy, Clone)]
pub enum Target {
  Android,
  #[cfg(target_os = "macos")]
  Ios,
}

impl Target {
  fn ide_name(&self) -> &'static str {
    match self {
      Self::Android => "Android Studio",
      #[cfg(target_os = "macos")]
      Self::Ios => "Xcode",
    }
  }

  fn command_name(&self) -> &'static str {
    match self {
      Self::Android => "android",
      #[cfg(target_os = "macos")]
      Self::Ios => "ios",
    }
  }

  fn ide_build_script_name(&self) -> &'static str {
    match self {
      Self::Android => "android-studio-script",
      #[cfg(target_os = "macos")]
      Self::Ios => "xcode-script",
    }
  }

  fn platform_target(&self) -> tauri_utils::platform::Target {
    match self {
      Self::Android => tauri_utils::platform::Target::Android,
      #[cfg(target_os = "macos")]
      Self::Ios => tauri_utils::platform::Target::Ios,
    }
  }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetDevice {
  id: String,
  name: String,
}

#[derive(Debug, Clone)]
pub struct DevHost(Option<Option<IpAddr>>);

impl FromStr for DevHost {
  type Err = AddrParseError;
  fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
    if s.is_empty() || s == "<public network address>" {
      Ok(Self(Some(None)))
    } else if s == "<none>" {
      Ok(Self(None))
    } else {
      IpAddr::from_str(s).map(|addr| Self(Some(Some(addr))))
    }
  }
}

impl Display for DevHost {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self.0 {
      Some(None) => write!(f, "<public network address>"),
      Some(Some(addr)) => write!(f, "{addr}"),
      None => write!(f, "<none>"),
    }
  }
}

impl Default for DevHost {
  fn default() -> Self {
    // on Windows we want to force using the public network address for the development server
    // because the adb port forwarding does not work well
    if cfg!(windows) {
      Self(Some(None))
    } else {
      Self(None)
    }
  }
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct CliOptions {
  pub dev: bool,
  pub features: Vec<String>,
  pub args: Vec<String>,
  pub noise_level: NoiseLevel,
  pub vars: HashMap<String, OsString>,
  pub config: Vec<ConfigValue>,
  pub target_device: Option<TargetDevice>,
}

fn local_ip_address(force: bool) -> &'static IpAddr {
  static LOCAL_IP: OnceLock<IpAddr> = OnceLock::new();
  LOCAL_IP.get_or_init(|| {
    let prompt_for_ip = || {
      let addresses: Vec<IpAddr> = local_ip_address::list_afinet_netifas()
        .expect("failed to list networks")
        .into_iter()
        .map(|(_, ipaddr)| ipaddr)
        .filter(|ipaddr| match ipaddr {
          IpAddr::V4(i) => i != &Ipv4Addr::LOCALHOST,
          IpAddr::V6(i) => i.to_string().ends_with("::2"),

        })
        .collect();
      match addresses.as_slice() {
        [] => panic!("No external IP detected."),
        [ipaddr] => *ipaddr,
        _ => {
          let selected = dialoguer::Select::with_theme(&dialoguer::theme::ColorfulTheme::default())
            .with_prompt(
              "Failed to detect external IP, What IP should we use to access your development server?",
            )
            .items(&addresses)
            .default(0)
            .interact()
            .expect("failed to select external IP");
          *addresses.get(selected).unwrap()
        }
      }
    };

    let ip = if force {
      prompt_for_ip()
    } else {
      local_ip_address::local_ip().unwrap_or_else(|_| prompt_for_ip())
    };
    log::info!("Using {ip} to access the development server.");
    ip
  })
}

struct DevUrlConfig {
  no_dev_server_wait: bool,
}

fn use_network_address_for_dev_url(
  config: &mut ConfigMetadata,
  dev_options: &mut crate::dev::Options,
  force_ip_prompt: bool,
  tauri_dir: &Path,
) -> crate::Result<DevUrlConfig> {
  let mut dev_url = config.build.dev_url.clone();

  let ip = if let Some(url) = &mut dev_url {
    let localhost = match url.host() {
      Some(url::Host::Domain(d)) => d == "localhost",
      Some(url::Host::Ipv4(i)) => i == Ipv4Addr::LOCALHOST || i == Ipv4Addr::UNSPECIFIED,
      _ => false,
    };

    if localhost {
      let ip = dev_options
        .host
        .unwrap_or_else(|| *local_ip_address(force_ip_prompt));
      log::info!(
        "Replacing devUrl host with {ip}. {}.",
        "If your frontend is not listening on that address, try configuring your development server to use the `TAURI_DEV_HOST` environment variable or 0.0.0.0 as host"
      );

      let url_str = format!(
        "{}://{}{}",
        url.scheme(),
        SocketAddr::new(ip, url.port_or_known_default().unwrap()),
        url.path()
      );
      *url =
        url::Url::parse(&url_str).with_context(|| format!("failed to parse URL: {url_str}"))?;

      dev_options
        .config
        .push(crate::ConfigValue(serde_json::json!({
          "build": {
            "devUrl": url
          }
        })));

      reload_config(
        config,
        &dev_options
          .config
          .iter()
          .map(|conf| &conf.0)
          .collect::<Vec<_>>(),
        tauri_dir,
      )?;

      Some(ip)
    } else {
      None
    }
  } else if !dev_options.no_dev_server {
    let ip = dev_options
      .host
      .unwrap_or_else(|| *local_ip_address(force_ip_prompt));
    dev_options.host.replace(ip);
    Some(ip)
  } else {
    None
  };

  let mut dev_url_config = DevUrlConfig {
    no_dev_server_wait: false,
  };

  if let Some(ip) = ip {
    unsafe { std::env::set_var("TAURI_DEV_HOST", ip.to_string()) };
    unsafe { std::env::set_var("TRUNK_SERVE_ADDRESS", ip.to_string()) };
    if ip.is_ipv6() {
      // in this case we can't ping the server for some reason
      dev_url_config.no_dev_server_wait = true;
    }
  }

  Ok(dev_url_config)
}

fn env_vars() -> HashMap<String, OsString> {
  let mut vars = HashMap::new();
  vars.insert("RUST_LOG_STYLE".into(), "always".into());
  for (k, v) in std::env::vars_os() {
    let k = k.to_string_lossy();
    if (k.starts_with("TAURI")
      && k != "TAURI_SIGNING_PRIVATE_KEY"
      && k != "TAURI_SIGNING_PRIVATE_KEY_PASSWORD")
      || k.starts_with("WRY")
      || k.starts_with("CARGO_")
      || k.starts_with("RUST_")
      || k == "TMPDIR"
      || k == "PATH"
    {
      vars.insert(k.into_owned(), v);
    }
  }
  vars
}

/// Environment variable name fragments that are never sent to the IDE build scripts
/// through the options server, since those variables usually hold secrets.
const SECRET_ENV_VAR_FRAGMENTS: &[&str] = &[
  "TOKEN",
  "PASSWORD",
  "PASSWD",
  "PASSPHRASE",
  "SECRET",
  "CREDENTIAL",
  "API_KEY",
  "ACCESS_KEY",
  "PRIVATE_KEY",
  "SIGNING_KEY",
  "RPM_KEY",
];

fn is_secret_env_var(name: &str) -> bool {
  let name = name.to_ascii_uppercase();
  SECRET_ENV_VAR_FRAGMENTS
    .iter()
    .any(|fragment| name.contains(fragment))
}

fn env() -> std::result::Result<Env, EnvError> {
  let env = Env::new()?.explicit_env_vars(env_vars());
  Ok(env)
}

/// JSON-RPC error code returned when the options request carries an invalid token.
const INVALID_TOKEN_ERROR_CODE: i32 = -32001;

/// Connection details of the options server, stored in [`options_server_file`].
#[derive(Serialize, Deserialize)]
struct OptionsServerInfo {
  addr: SocketAddr,
  token: String,
}

/// Path of the file the `dev` and `build` commands use to share the options server details
/// with the Xcode and Android Studio build scripts.
fn options_server_file(target: Target, tauri_dir: &Path) -> PathBuf {
  let project_dir = match target {
    Target::Android => "android",
    #[cfg(target_os = "macos")]
    Target::Ios => "apple",
  };
  tauri_dir
    .join("gen")
    .join(project_dir)
    .join(".tauri")
    .join("cli-options-server.json")
}

fn write_options_server_file(path: &Path, contents: &str) -> Result<()> {
  let dir = path
    .parent()
    .context("options server file has no parent directory")?;
  create_dir_all(dir).fs_context("failed to create directory", dir.to_path_buf())?;
  let gitignore = dir.join(".gitignore");
  if !gitignore.exists() {
    std::fs::write(&gitignore, "*\n").fs_context("failed to write .gitignore", gitignore)?;
  }

  // never write through a stale file or symlink left at this path
  if path.symlink_metadata().is_ok() {
    remove_file(path).fs_context(
      "failed to remove stale options server file",
      path.to_path_buf(),
    )?;
  }

  let mut open_options = OpenOptions::new();
  open_options.write(true).create_new(true);
  #[cfg(unix)]
  {
    use std::os::unix::fs::OpenOptionsExt;
    open_options.mode(0o600);
  }
  open_options
    .open(path)
    .and_then(|mut file| file.write_all(contents.as_bytes()))
    .fs_context("failed to write options server file", path.to_path_buf())
}

/// Compares two tokens in constant time (for tokens of the same length).
fn token_matches(expected: &str, provided: &str) -> bool {
  let (expected, provided) = (expected.as_bytes(), provided.as_bytes());
  expected.len() == provided.len()
    && expected
      .iter()
      .zip(provided)
      .fold(0u8, |acc, (a, b)| acc | (a ^ b))
      == 0
}

/// HTTP middleware that rejects requests carrying an `Origin` header.
///
/// Browsers always send it on WebSocket upgrades, so web pages can't reach the options server,
/// while the CLI client started by the IDE build scripts never sends it.
#[derive(Clone)]
struct RejectOrigin<S>(S);

impl<S> tower::Service<HttpRequest> for RejectOrigin<S>
where
  S: tower::Service<HttpRequest, Response = HttpResponse, Error = BoxError>,
  S::Future: Send + 'static,
{
  type Response = HttpResponse;
  type Error = BoxError;
  type Future = Pin<Box<dyn Future<Output = std::result::Result<HttpResponse, BoxError>> + Send>>;

  fn poll_ready(
    &mut self,
    cx: &mut std::task::Context<'_>,
  ) -> Poll<std::result::Result<(), BoxError>> {
    self.0.poll_ready(cx)
  }

  fn call(&mut self, request: HttpRequest) -> Self::Future {
    if request.headers().contains_key("origin") {
      Box::pin(std::future::ready(Ok(
        jsonrpsee::server::http::response::denied(),
      )))
    } else {
      Box::pin(self.0.call(request))
    }
  }
}

pub struct OptionsHandle {
  _runtime: Runtime,
  _server: ServerHandle,
  server_file: PathBuf,
  server_file_contents: String,
}

impl Drop for OptionsHandle {
  fn drop(&mut self) {
    // leave the file alone if another CLI session replaced it
    if read_to_string(&self.server_file).is_ok_and(|contents| contents == self.server_file_contents)
    {
      let _ = remove_file(&self.server_file);
    }
  }
}

/// Writes CLI options to be used later on the Xcode and Android Studio build commands
pub fn write_options(
  target: Target,
  tauri_dir: &Path,
  mut options: CliOptions,
) -> crate::Result<OptionsHandle> {
  options.vars.extend(env_vars());
  options.vars.retain(|name, _| !is_secret_env_var(name));

  let token = Alphanumeric.sample_string(&mut rand::rng(), 32);
  let server_token = token.clone();

  let runtime = Runtime::new().context("failed to create async runtime")?;
  let r: crate::Result<(ServerHandle, SocketAddr)> = runtime.block_on(async move {
    let server = ServerBuilder::default()
      .set_http_middleware(tower::ServiceBuilder::new().layer_fn(RejectOrigin))
      .build("127.0.0.1:0")
      .await
      .context("failed to build WebSocket server")?;
    let addr = server.local_addr().context("failed to get local address")?;

    let mut module = RpcModule::new(());
    module
      .register_method("options", move |params, _, _| {
        let token: String = params.one()?;
        if token_matches(&server_token, &token) {
          Ok(options.clone())
        } else {
          Err(ErrorObjectOwned::owned(
            INVALID_TOKEN_ERROR_CODE,
            "invalid options server token",
            None::<()>,
          ))
        }
      })
      .context("failed to register options method")?;

    let handle = server.start(module);

    Ok((handle, addr))
  });
  let (handle, addr) = r?;

  let server_file = options_server_file(target, tauri_dir);
  let server_file_contents = serde_json::to_string(&OptionsServerInfo { addr, token })
    .context("failed to serialize options server details")?;
  write_options_server_file(&server_file, &server_file_contents)?;

  Ok(OptionsHandle {
    _runtime: runtime,
    _server: handle,
    server_file,
    server_file_contents,
  })
}

/// Requests the CLI options from the `dev` or `build` command that started the IDE build.
fn fetch_options(target: Target, tauri_dir: &Path) -> Result<CliOptions> {
  let not_running = move || {
    format!(
      "the `tauri {0} dev` or `tauri {0} build` command must be running while {1} builds the app",
      target.command_name(),
      target.ide_name()
    )
  };

  let server_file = options_server_file(target, tauri_dir);
  let contents = read_to_string(&server_file).with_context(|| {
    format!(
      "failed to read {}; {}",
      server_file.display(),
      not_running()
    )
  })?;
  let info: OptionsServerInfo = serde_json::from_str(&contents)
    .with_context(|| format!("failed to parse {}", server_file.display()))?;

  let runtime = Runtime::new().context("failed to create async runtime")?;
  runtime.block_on(async move {
    let url = format!("ws://{}", info.addr)
      .parse()
      .context("failed to parse options server URL")?;
    let (tx, rx) = WsTransportClientBuilder::default()
      .build(url)
      .await
      .with_context(|| format!("failed to connect to the Tauri CLI; {}", not_running()))?;
    let client: Client = ClientBuilder::default().build_with_tokio(tx, rx);
    client
      .request("options", rpc_params![info.token])
      .await
      .context("failed to request options from the Tauri CLI")
  })
}

fn read_options(target: Target, tauri_dir: &Path) -> Result<CliOptions> {
  let options = fetch_options(target, tauri_dir)?;
  for (k, v) in &options.vars {
    unsafe { set_var(k, v) };
  }
  Ok(options)
}

pub fn get_app(
  target: Target,
  config: &TauriConfig,
  interface: &AppInterface,
  tauri_dir: &Path,
) -> App {
  let identifier = match target {
    Target::Android => config.identifier.replace('-', "_"),
    #[cfg(target_os = "macos")]
    Target::Ios => config.identifier.replace('_', "-"),
  };

  if identifier.is_empty() {
    log::error!("Bundle identifier set in `tauri.conf.json > identifier` cannot be empty");
    exit(1);
  }

  let app_name = interface
    .app_settings()
    .app_name()
    .unwrap_or_else(|| "app".into());
  let lib_name = interface
    .app_settings()
    .lib_name()
    .unwrap_or_else(|| app_name.to_snek_case());

  if config.product_name.is_none() {
    log::warn!(
      "`productName` is not set in the Tauri configuration. Using `{app_name}` as the app name."
    );
  }

  let raw = RawAppConfig {
    name: app_name,
    lib_name: Some(lib_name),
    stylized_name: config.product_name.clone(),
    identifier,
    asset_dir: None,
    template_pack: None,
  };

  let app_settings = interface.app_settings();
  let tauri_dir = tauri_dir.to_path_buf();
  App::from_raw(tauri_dir.to_path_buf(), raw)
    .unwrap()
    .with_target_dir_resolver(move |target, profile| {
      app_settings
        .out_dir(
          &InterfaceOptions {
            debug: matches!(profile, Profile::Debug),
            target: Some(target.into()),
            ..Default::default()
          },
          &tauri_dir,
        )
        .expect("failed to resolve target directory")
    })
}

#[allow(unused_variables)]
fn ensure_init(
  tauri_config: &ConfigMetadata,
  app: &App,
  project_dir: PathBuf,
  target: Target,
  noninteractive: bool,
) -> Result<()> {
  if !project_dir.exists() {
    crate::error::bail!(
      "{} project directory {} doesn't exist. Please run `tauri {} init` and try again.",
      target.ide_name(),
      project_dir.display(),
      target.command_name(),
    )
  }

  let mut project_outdated_reasons = Vec::new();

  match target {
    Target::Android => {
      let java_folder = project_dir
        .join("app/src/main/java")
        .join(tauri_config.identifier.replace('.', "/").replace('-', "_"));
      if java_folder.exists() {
        ensure_gradlew(&project_dir)?;
      } else {
        project_outdated_reasons
          .push("you have modified your \"identifier\" in the Tauri configuration");
      }
    }
    #[cfg(target_os = "macos")]
    Target::Ios => {
      let xcodeproj_path = crate::helpers::fs::find_in_directory(&project_dir, "*.xcodeproj")
        .with_context(|| format!("failed to locate xcodeproj in {}", project_dir.display()))?;

      let xcodeproj_name = xcodeproj_path.file_stem().unwrap().to_str().unwrap();
      if xcodeproj_name != app.name() {
        let rename_targets = vec![
          // first rename the entitlements
          (
            format!("{xcodeproj_name}_iOS/{xcodeproj_name}_iOS.entitlements"),
            format!("{xcodeproj_name}_iOS/{}_iOS.entitlements", app.name()),
          ),
          // then the scheme folder
          (
            format!("{xcodeproj_name}_iOS"),
            format!("{}_iOS", app.name()),
          ),
          (
            format!("{xcodeproj_name}.xcodeproj"),
            format!("{}.xcodeproj", app.name()),
          ),
        ];
        let rename_info = rename_targets
          .iter()
          .map(|(from, to)| format!("- {from} to {to}"))
          .collect::<Vec<_>>()
          .join("\n");
        log::error!(
          "you have modified your package name from {current_project_name} to {new_project_name}\nWe need to apply the name change to the Xcode project, renaming:\n{rename_info}",
          new_project_name = app.name(),
          current_project_name = xcodeproj_name,
        );
        if noninteractive {
          project_outdated_reasons
            .push("you have modified your [lib.name] or [package.name] in the Cargo.toml file");
        } else {
          let confirm = crate::helpers::prompts::confirm(
            "Do you want to apply the name change to the Xcode project?",
            Some(true),
          )
          .unwrap_or_default();
          if confirm {
            for (from, to) in rename_targets {
              std::fs::rename(project_dir.join(&from), project_dir.join(&to))
                .with_context(|| format!("failed to rename {from} to {to}"))?;
            }

            // update scheme name in pbxproj
            // identifier / product name are synchronized by the dev/build commands
            let pbxproj_path =
              project_dir.join(format!("{}.xcodeproj/project.pbxproj", app.name()));
            let pbxproj_contents = std::fs::read_to_string(&pbxproj_path)
              .with_context(|| format!("failed to read {}", pbxproj_path.display()))?;
            std::fs::write(
              &pbxproj_path,
              pbxproj_contents.replace(
                &format!("{xcodeproj_name}_iOS"),
                &format!("{}_iOS", app.name()),
              ),
            )
            .with_context(|| format!("failed to write {}", pbxproj_path.display()))?;
          } else {
            project_outdated_reasons
              .push("you have modified your [lib.name] or [package.name] in the Cargo.toml file");
          }
        }
      }

      // note: pbxproj is synchronied by the dev/build commands
    }
  }

  if !project_outdated_reasons.is_empty() {
    let reason = project_outdated_reasons.join(" and ");
    crate::error::bail!(
      "{} project directory is outdated because {reason}. Please delete {}, run `tauri {} init` and try again.",
      target.ide_name(),
      project_dir.display(),
      target.command_name(),
    )
  }

  Ok(())
}

fn ensure_gradlew(project_dir: &std::path::Path) -> Result<()> {
  let gradlew_path = project_dir.join("gradlew");

  #[cfg(unix)]
  {
    use std::os::unix::fs::PermissionsExt;

    if let Ok(metadata) = gradlew_path.metadata() {
      let mut permissions = metadata.permissions();
      let is_executable = permissions.mode() & 0o111 != 0;
      if !is_executable {
        permissions.set_mode(permissions.mode() | 0o111);
        std::fs::set_permissions(&gradlew_path, permissions)
          .fs_context("failed to mark gradlew as executable", &gradlew_path)?;
      }
    }
  }

  // A gradlew with CRLF line endings cannot run: sh fails with
  // "/usr/bin/env: 'sh\r': No such file or directory" or similar, which
  // also happens under Git Bash on Windows, so the rewrite runs on all
  // platforms (https://github.com/tauri-apps/tauri/pull/16017). Windows
  // builds invoke gradlew.bat, so there a gradlew that cannot be
  // rewritten only draws a warning; on unix the error is returned.
  if gradlew_path.exists() {
    let result = std::fs::read_to_string(&gradlew_path)
      .fs_context("failed to read gradlew", &gradlew_path)
      .and_then(|contents| {
        if contents.contains("\r\n") {
          std::fs::write(&gradlew_path, contents.replace("\r\n", "\n"))
            .fs_context("failed to replace gradlew CRLF with LF", &gradlew_path)
        } else {
          Ok(())
        }
      });
    #[cfg(unix)]
    result?;
    #[cfg(not(unix))]
    if let Err(error) = result {
      log::warn!("failed to normalize gradlew line endings: {error}");
    }
  }

  Ok(())
}

fn log_finished(outputs: Vec<PathBuf>, kind: &str) {
  if !outputs.is_empty() {
    let mut printable_paths = String::new();
    for path in &outputs {
      writeln!(printable_paths, "        {}", path.display()).unwrap();
    }

    log::info!(action = "Finished"; "{} {}{} at:\n{}", outputs.len(), kind, if outputs.len() == 1 { "" } else { "s" }, printable_paths);
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn detects_secret_env_vars() {
    for name in [
      "CARGO_TARGET_DIR",
      "CARGO_PKG_AUTHORS",
      "TAURI_DEV_HOST",
      "TAURI_DEV_ROOT_CERTIFICATE",
      "RUST_LOG",
      "PATH",
    ] {
      assert!(!is_secret_env_var(name), "{name} is not a secret");
    }
    for name in [
      "CARGO_REGISTRY_TOKEN",
      "CARGO_REGISTRIES_MY_REGISTRY_TOKEN",
      "CARGO_REGISTRY_CREDENTIAL_PROVIDER",
      "TAURI_SIGNING_PRIVATE_KEY_PASSWORD",
      "TAURI_CLOUD_Secret",
      "RUST_api_token",
      "APPLE_API_KEY",
      "APPLE_API_KEY_PATH",
      "APPLE_CERTIFICATE_PASSWORD",
      "TAURI_PRIVATE_KEY",
      "TAURI_SIGNING_PRIVATE_KEY_PATH",
      "TAURI_SIGNING_RPM_KEY",
      "TAURI_SIGNING_RPM_KEY_PASSPHRASE",
      "AWS_SECRET_ACCESS_KEY",
      "AWS_ACCESS_KEY_ID",
      "ANDROID_KEYSTORE_PASSWD",
    ] {
      assert!(is_secret_env_var(name), "{name} is a secret");
    }
  }

  #[test]
  fn options_server_requires_token_and_rejects_origin() {
    let tauri_dir = tempfile::tempdir().unwrap();
    let target = Target::Android;
    let server_file = options_server_file(target, tauri_dir.path());

    let handle = write_options(
      target,
      tauri_dir.path(),
      CliOptions {
        args: vec!["--test-arg".into()],
        vars: HashMap::from([
          ("CARGO_REGISTRY_TOKEN".into(), "secret".into()),
          ("TAURI_TEST_VAR".into(), "value".into()),
        ]),
        ..Default::default()
      },
    )
    .unwrap();

    #[cfg(unix)]
    {
      use std::os::unix::fs::PermissionsExt;
      let mode = server_file.metadata().unwrap().permissions().mode();
      assert_eq!(mode & 0o777, 0o600);
    }

    let options = fetch_options(target, tauri_dir.path()).unwrap();
    assert_eq!(options.args, vec!["--test-arg".to_string()]);
    assert_eq!(
      options.vars.get("TAURI_TEST_VAR"),
      Some(&OsString::from("value"))
    );
    assert!(!options.vars.contains_key("CARGO_REGISTRY_TOKEN"));

    let info: OptionsServerInfo =
      serde_json::from_str(&read_to_string(&server_file).unwrap()).unwrap();
    let runtime = Runtime::new().unwrap();
    runtime.block_on(async {
      let (tx, rx) = WsTransportClientBuilder::default()
        .build(format!("ws://{}", info.addr).parse().unwrap())
        .await
        .unwrap();
      let client: Client = ClientBuilder::default().build_with_tokio(tx, rx);
      let wrong_token = "x".repeat(info.token.len());
      assert!(
        client
          .request::<CliOptions, _>("options", rpc_params![wrong_token])
          .await
          .is_err()
      );
      assert!(
        client
          .request::<CliOptions, _>("options", rpc_params![])
          .await
          .is_err()
      );

      // browsers always send an Origin header on WebSocket upgrades
      let mut headers = jsonrpsee_client_transport::ws::HeaderMap::new();
      headers.insert("origin", "https://example.com".parse().unwrap());
      assert!(
        WsTransportClientBuilder::default()
          .set_headers(headers)
          .build(format!("ws://{}", info.addr).parse().unwrap())
          .await
          .is_err()
      );
    });

    drop(handle);
    assert!(!server_file.exists());
    assert!(fetch_options(target, tauri_dir.path()).is_err());
  }

  #[cfg(unix)]
  #[test]
  fn options_server_file_does_not_follow_symlinks() {
    let dir = tempfile::tempdir().unwrap();
    let victim = dir.path().join("victim");
    std::fs::write(&victim, "untouched").unwrap();
    let path = dir.path().join(".tauri").join("cli-options-server.json");
    create_dir_all(path.parent().unwrap()).unwrap();
    std::os::unix::fs::symlink(&victim, &path).unwrap();

    write_options_server_file(&path, "contents").unwrap();

    assert_eq!(read_to_string(&victim).unwrap(), "untouched");
    assert!(!path.symlink_metadata().unwrap().file_type().is_symlink());
    assert_eq!(read_to_string(&path).unwrap(), "contents");
  }
}
