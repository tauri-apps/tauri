// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{
  helpers::{
    app_paths::tauri_dir,
    config::{reload as reload_config, Config as TauriConfig, ConfigHandle},
  },
  interface::{AppInterface, AppSettings, DevProcess, Interface, Options as InterfaceOptions},
  ConfigValue,
};
#[cfg(unix)]
use anyhow::Context;
use anyhow::{bail, Result};
use heck::ToSnekCase;
use jsonrpsee::core::client::{Client, ClientBuilder, ClientT};
use jsonrpsee::server::{RpcModule, ServerBuilder, ServerHandle};
use jsonrpsee_client_transport::ws::WsTransportClientBuilder;
use jsonrpsee_core::rpc_params;
use serde::{Deserialize, Serialize};

use cargo_mobile2::{
  config::app::{App, Raw as RawAppConfig},
  env::Error as EnvError,
  opts::{NoiseLevel, Profile},
  ChildHandle,
};
use std::{
  collections::HashMap,
  env::{set_var, temp_dir},
  ffi::OsString,
  fmt::{Display, Write},
  fs::{read_to_string, write},
  net::{AddrParseError, IpAddr, Ipv4Addr, SocketAddr},
  path::PathBuf,
  process::{exit, ExitStatus},
  str::FromStr,
  sync::{
    atomic::{AtomicBool, Ordering},
    Arc, OnceLock,
  },
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
    self.manually_killed_process.store(true, Ordering::Relaxed);
    match self.child.kill() {
      Ok(_) => Ok(()),
      Err(e) => {
        self.manually_killed_process.store(false, Ordering::Relaxed);
        Err(e)
      }
    }
  }

  fn try_wait(&self) -> std::io::Result<Option<ExitStatus>> {
    self.child.try_wait().map(|res| res.map(|o| o.status))
  }

  fn wait(&self) -> std::io::Result<ExitStatus> {
    self.child.wait().map(|o| o.status)
  }

  fn manually_killed_process(&self) -> bool {
    self.manually_killed_process.load(Ordering::Relaxed)
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CliOptions {
  pub dev: bool,
  pub features: Option<Vec<String>>,
  pub args: Vec<String>,
  pub noise_level: NoiseLevel,
  pub vars: HashMap<String, OsString>,
  pub config: Option<ConfigValue>,
  pub target_device: Option<TargetDevice>,
}

impl Default for CliOptions {
  fn default() -> Self {
    Self {
      dev: false,
      features: None,
      args: vec!["--lib".into()],
      noise_level: Default::default(),
      vars: Default::default(),
      config: None,
      target_device: None,
    }
  }
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
      match addresses.len() {
        0 => panic!("No external IP detected."),
        1 => {
          let ipaddr = addresses.first().unwrap();
          *ipaddr
        }
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
  config: &ConfigHandle,
  dev_options: &mut crate::dev::Options,
  force_ip_prompt: bool,
) -> crate::Result<DevUrlConfig> {
  let mut dev_url = config
    .lock()
    .unwrap()
    .as_ref()
    .unwrap()
    .build
    .dev_url
    .clone();

  let ip = if let Some(url) = &mut dev_url {
    let localhost = match url.host() {
      Some(url::Host::Domain(d)) => d == "localhost",
      Some(url::Host::Ipv4(i)) => {
        i == std::net::Ipv4Addr::LOCALHOST || i == std::net::Ipv4Addr::UNSPECIFIED
      }
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

      *url = url::Url::parse(&format!(
        "{}://{}{}",
        url.scheme(),
        SocketAddr::new(ip, url.port_or_known_default().unwrap()),
        url.path()
      ))?;

      if let Some(c) = &mut dev_options.config {
        if let Some(build) = c
          .0
          .as_object_mut()
          .and_then(|root| root.get_mut("build"))
          .and_then(|build| build.as_object_mut())
        {
          build.insert("devUrl".into(), url.to_string().into());
        }
      } else {
        let mut build = serde_json::Map::new();
        build.insert("devUrl".into(), url.to_string().into());

        dev_options
          .config
          .replace(crate::ConfigValue(serde_json::json!({
            "build": build
          })));
      }
      reload_config(dev_options.config.as_ref().map(|c| &c.0))?;

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
    std::env::set_var("TAURI_DEV_HOST", ip.to_string());
    std::env::set_var("TRUNK_SERVE_ADDRESS", ip.to_string());
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
      || k == "TMPDIR"
      || k == "PATH"
    {
      vars.insert(k.into_owned(), v);
    }
  }
  vars
}

fn env() -> Result<Env, EnvError> {
  let env = Env::new()?.explicit_env_vars(env_vars());
  Ok(env)
}

pub struct OptionsHandle(#[allow(unused)] Runtime, #[allow(unused)] ServerHandle);

/// Writes CLI options to be used later on the Xcode and Android Studio build commands
pub fn write_options(identifier: &str, mut options: CliOptions) -> crate::Result<OptionsHandle> {
  options.vars.extend(env_vars());

  let runtime = Runtime::new().unwrap();
  let r: anyhow::Result<(ServerHandle, SocketAddr)> = runtime.block_on(async move {
    let server = ServerBuilder::default().build("127.0.0.1:0").await?;
    let addr = server.local_addr()?;

    let mut module = RpcModule::new(());
    module.register_method("options", move |_, _, _| Some(options.clone()))?;

    let handle = server.start(module);

    Ok((handle, addr))
  });
  let (handle, addr) = r?;

  write(
    temp_dir().join(format!("{identifier}-server-addr")),
    addr.to_string(),
  )?;

  Ok(OptionsHandle(runtime, handle))
}

fn read_options(identifier: &str) -> CliOptions {
  let runtime = tokio::runtime::Runtime::new().unwrap();
  let options = runtime
    .block_on(async move {
      let addr_path = temp_dir().join(format!("{identifier}-server-addr"));
      let (tx, rx) = WsTransportClientBuilder::default()
        .build(
          format!(
            "ws://{}",
            read_to_string(&addr_path).unwrap_or_else(|e| panic!(
              "failed to read missing addr file {}: {e}",
              addr_path.display()
            ))
          )
          .parse()
          .unwrap(),
        )
        .await?;
      let client: Client = ClientBuilder::default().build_with_tokio(tx, rx);
      let options: CliOptions = client.request("options", rpc_params![]).await?;
      Ok::<CliOptions, anyhow::Error>(options)
    })
    .expect("failed to read CLI options");

  for (k, v) in &options.vars {
    set_var(k, v);
  }
  options
}

pub fn get_app(target: Target, config: &TauriConfig, interface: &AppInterface) -> App {
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

  let raw = RawAppConfig {
    name: app_name,
    lib_name: Some(lib_name),
    stylized_name: config.product_name.clone(),
    identifier,
    asset_dir: None,
    template_pack: None,
  };

  let app_settings = interface.app_settings();
  App::from_raw(tauri_dir().to_path_buf(), raw)
    .unwrap()
    .with_target_dir_resolver(move |target, profile| {
      app_settings
        .out_dir(&InterfaceOptions {
          debug: matches!(profile, Profile::Debug),
          target: Some(target.into()),
          ..Default::default()
        })
        .expect("failed to resolve target directory")
    })
}

#[allow(unused_variables)]
fn ensure_init(
  tauri_config: &ConfigHandle,
  app: &App,
  project_dir: PathBuf,
  target: Target,
) -> Result<()> {
  if !project_dir.exists() {
    bail!(
      "{} project directory {} doesn't exist. Please run `tauri {} init` and try again.",
      target.ide_name(),
      project_dir.display(),
      target.command_name(),
    )
  }

  let tauri_config_guard = tauri_config.lock().unwrap();
  let tauri_config_ = tauri_config_guard.as_ref().unwrap();

  let mut project_outdated_reasons = Vec::new();

  match target {
    Target::Android => {
      let java_folder = project_dir
        .join("app/src/main/java")
        .join(tauri_config_.identifier.replace('.', "/").replace('-', "_"));
      if java_folder.exists() {
        #[cfg(unix)]
        ensure_gradlew(&project_dir)?;
      } else {
        project_outdated_reasons
          .push("you have modified your \"identifier\" in the Tauri configuration");
      }
    }
    #[cfg(target_os = "macos")]
    Target::Ios => {
      let pbxproj_contents = read_to_string(
        project_dir
          .join(format!("{}.xcodeproj", app.name()))
          .join("project.pbxproj"),
      )
      .context("missing project.yml file in the Xcode project directory")?;

      if !(pbxproj_contents.contains(ios::LIB_OUTPUT_FILE_NAME)
        || pbxproj_contents.contains(&format!("lib{}.a", app.lib_name())))
      {
        project_outdated_reasons
          .push("you have modified your [lib.name] or [package.name] in the Cargo.toml file");
      }
    }
  }

  if !project_outdated_reasons.is_empty() {
    let reason = project_outdated_reasons.join(" and ");
    bail!(
        "{} project directory is outdated because {reason}. Please run `tauri {} init` and try again.",
        target.ide_name(),
        target.command_name(),
      )
  }

  Ok(())
}

#[cfg(unix)]
fn ensure_gradlew(project_dir: &std::path::Path) -> Result<()> {
  use std::os::unix::fs::PermissionsExt;

  let gradlew_path = project_dir.join("gradlew");
  if let Ok(metadata) = gradlew_path.metadata() {
    let mut permissions = metadata.permissions();
    let is_executable = permissions.mode() & 0o111 != 0;
    if !is_executable {
      permissions.set_mode(permissions.mode() | 0o111);
      std::fs::set_permissions(&gradlew_path, permissions)
        .context("failed to mark gradlew as executable")?;
    }
    std::fs::write(
      &gradlew_path,
      std::fs::read_to_string(&gradlew_path)
        .context("failed to read gradlew")?
        .replace("\r\n", "\n"),
    )
    .context("failed to replace gradlew CRLF with LF")?;
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
