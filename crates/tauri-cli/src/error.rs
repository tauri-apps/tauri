// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use std::{borrow::Cow, fmt::Display, path::PathBuf};

#[derive(Debug, thiserror::Error)]
pub enum Error {
  #[error("{0}: {1}")]
  Context(String, Box<Self>),
  #[error("{0}")]
  GenericError(String),
  #[error("failed to bundle project {0}")]
  Bundler(#[from] tauri_bundler::Error),
  #[error("failed to run command {command}: {error}")]
  CommandFailed {
    command: String,
    error: std::io::Error,
  },
  #[error("{context}: {error}")]
  ParseConfig {
    context: &'static str,
    error: tauri_utils::config::parse::ConfigError,
  },
  #[error("failed to kill {name}: {error}")]
  KillProcess {
    name: &'static str,
    error: std::io::Error,
  },
  #[error("{context} {path}: {error}")]
  Fs {
    context: &'static str,
    path: PathBuf,
    error: std::io::Error,
  },
  #[error("failed to resolve current directory {0}")]
  ResolveCwd(std::io::Error),
  #[error("failed to set current directory {0}")]
  SetCwd(std::io::Error),
  #[error("{context}: {error}")]
  Json {
    context: Cow<'static, str>,
    error: serde_json::Error,
  },
  #[error("{context}: {error}")]
  Json5 {
    context: Cow<'static, str>,
    error: json5::Error,
  },
  #[error("{context}: {error}")]
  DeserializeToml {
    context: Cow<'static, str>,
    error: toml::de::Error,
  },
  #[error("{context}: {error}")]
  DeserializeTomlEdit {
    context: Cow<'static, str>,
    error: toml_edit::TomlError,
  },
  #[error("{context}: {error}")]
  SerializeToml {
    context: Cow<'static, str>,
    error: toml::ser::Error,
  },
  #[error("{context}: {error}")]
  SerializeTomlEdit {
    context: Cow<'static, str>,
    error: toml_edit::ser::Error,
  },
  #[error("failed to run glob: {pattern}: {error}")]
  Glob {
    pattern: String,
    error: glob::GlobError,
  },
  #[error("failed to parse glob pattern `{pattern}`: {error}")]
  GlobPattern {
    pattern: String,
    error: glob::PatternError,
  },
  #[error("failed to parse url `{url}`: {error}")]
  ParseUrl { url: String, error: url::ParseError },
  #[error("failed to setup ws server: {0}")]
  SetupWsServer(std::io::Error),
  #[error("failed to connect to server: {0}")]
  WsHandshake(#[from] jsonrpsee_client_transport::ws::WsHandshakeError),
  #[error("failed to send request: {0}")]
  WsClient(#[from] jsonrpsee_core::client::Error),
  #[error("failed to register method: {0}")]
  RegisterMethod(#[from] jsonrpsee_core::RegisterMethodError),
  #[cfg(target_os = "macos")]
  #[error(transparent)]
  MacosSign(#[from] tauri_macos_sign::Error),
  #[error("resource error: {0}")]
  Resource(tauri_utils::Error),
  #[error("failed to prompt for mobile simulator: {0}")]
  PromptSimulator(std::io::Error),
  #[error("failed to start mobile simulator: {0}")]
  StartSimulator(std::io::Error),
  #[error("failed to prompt for mobile device: {0}")]
  PromptDevice(std::io::Error),
  #[cfg(target_os = "macos")]
  #[error("failed to initialize Apple config: {0}")]
  AppleConfig(cargo_mobile2::apple::config::Error),
  #[cfg(target_os = "macos")]
  #[error("failed to compile iOS library: {0}")]
  CompileIosLibrary(#[from] cargo_mobile2::apple::target::CompileLibError),
  #[cfg(target_os = "macos")]
  #[error("failed to build iOS app: {0}")]
  BuildIosApp(#[from] cargo_mobile2::apple::target::BuildError),
  #[cfg(target_os = "macos")]
  #[error("failed to archive iOS app: {0}")]
  ArchiveIosApp(#[from] cargo_mobile2::apple::target::ArchiveError),
  #[cfg(target_os = "macos")]
  #[error("failed to export iOS app: {0}")]
  ExportIosApp(#[from] cargo_mobile2::apple::target::ExportError),
  #[error("failed to build Android app: {0}")]
  BuildAndroidApp(#[from] cargo_mobile2::android::target::BuildError),
  #[error("failed to build AAB: {0}")]
  BuildAab(#[from] cargo_mobile2::android::aab::AabError),
  #[error("failed to build APK: {0}")]
  BuildApk(#[from] cargo_mobile2::android::apk::ApkError),
  #[error("failed to initialize mobile environment: {0}")]
  MobileEnv(#[from] cargo_mobile2::env::Error),
  #[error("failed to initialize Android environment: {0}")]
  AndroidEnv(#[from] cargo_mobile2::android::env::Error),
  #[cfg(target_os = "macos")]
  #[error("failed to run IOS app: {0}")]
  IosRun(#[from] cargo_mobile2::apple::device::RunError),
  #[error("failed to run Android app: {0}")]
  AndroidRun(#[from] cargo_mobile2::android::device::RunError),
  #[error("failed to create temp file: {0}")]
  TempFile(std::io::Error),
  #[error("failed to create temp dir: {0}")]
  TempDir(std::io::Error),
  #[error("failed to render template: {0}")]
  Template(#[from] handlebars::RenderError),
  #[error("failed to find Android tool: {0}")]
  MissingAndroidTool(#[from] cargo_mobile2::android::ndk::MissingToolError),
  #[error("failed to parse semver version: {version}: {error}")]
  ParseSemver {
    version: String,
    error: semver::Error,
  },
  #[error("{context}: {error}")]
  MagicString {
    context: Cow<'static, str>,
    error: magic_string::Error,
  },
  #[error("failed to parse ELF: {0}")]
  ParseElf(#[from] elf::ParseError),
  #[error("failed to watch {path}: {error}")]
  Watch { path: PathBuf, error: notify::Error },
  #[error("failed to get target triple: {0}")]
  TargetTriple(tauri_utils::Error),
  #[error("{context}: {error}")]
  Image {
    context: Cow<'static, str>,
    error: image::ImageError,
  },
  #[error("minisign error: {0}")]
  Minisign(#[from] minisign::PError),
  #[error("{context}: {error}")]
  Base64Decode {
    context: Cow<'static, str>,
    error: base64::DecodeError,
  },
  #[error("failed to convert base64 to string: {0}")]
  Base64NotUtf8(std::str::Utf8Error),
  #[error("prompt error: {0}")]
  Prompt(#[from] dialoguer::Error),
}

/// Convenient type alias of Result type.
pub type Result<T> = std::result::Result<T, Error>;

pub trait Context<T> {
  // Required methods
  fn context<C>(self, context: C) -> Result<T>
  where
    C: Display + Send + Sync + 'static;
  fn with_context<C, F>(self, f: F) -> Result<T>
  where
    C: Display + Send + Sync + 'static,
    F: FnOnce() -> C;
}

impl<T> Context<T> for Result<T> {
  fn context<C>(self, context: C) -> Result<T>
  where
    C: Display + Send + Sync + 'static,
  {
    self.map_err(|e| Error::Context(context.to_string(), Box::new(e)))
  }

  fn with_context<C, F>(self, f: F) -> Result<T>
  where
    C: Display + Send + Sync + 'static,
    F: FnOnce() -> C,
  {
    self.map_err(|e| Error::Context(f().to_string(), Box::new(e)))
  }
}

impl<T> Context<T> for Option<T> {
  fn context<C>(self, context: C) -> Result<T>
  where
    C: Display + Send + Sync + 'static,
  {
    self.ok_or_else(|| Error::GenericError(context.to_string()))
  }

  fn with_context<C, F>(self, f: F) -> Result<T>
  where
    C: Display + Send + Sync + 'static,
    F: FnOnce() -> C,
  {
    self.ok_or_else(|| Error::GenericError(f().to_string()))
  }
}

pub trait ErrorExt<T> {
  fn fs_context(self, context: &'static str, path: impl Into<PathBuf>) -> Result<T>;
}

impl<T> ErrorExt<T> for std::result::Result<T, std::io::Error> {
  fn fs_context(self, context: &'static str, path: impl Into<PathBuf>) -> Result<T> {
    self.map_err(|error| Error::Fs {
      context: context.into(),
      path: path.into(),
      error,
    })
  }
}

macro_rules! bail {
   ($msg:literal $(,)?) => {
      return Err(crate::Error::GenericError($msg.into()))
   };
    ($err:expr $(,)?) => {
       return Err(crate::Error::GenericError($err))
    };
   ($fmt:expr, $($arg:tt)*) => {
     return Err(crate::Error::GenericError(format!($fmt, $($arg)*)))
   };
}

pub(crate) use bail;
