use thiserror::Error as DeriveError;

#[derive(Debug, DeriveError)]
pub enum Error {
  /// IO Errors.
  #[error("`{0}`")]
  Io(#[from] std::io::Error),
  /// Reqwest Errors.
  #[error("Request Error: {0}")]
  Reqwest(#[from] reqwest::Error),
  /// Semver Errors.
  #[error("Unable to compare version: {0}")]
  Semver(#[from] semver::SemVerError),
  /// JSON (Serde) Errors.
  #[error("JSON Error: {0}")]
  SerdeJson(#[from] serde_json::Error),
  /// Minisign is used for signature validation.
  #[error("Verify signature Error: {0}")]
  Minisign(#[from] minisign_verify::Error),
  /// Error with Minisign base64 encoding.
  #[error("Signature decoding Error: {0}")]
  Base64(#[from] base64::DecodeError),
  /// UTF8 Errors in signature.
  #[error("Signature encoding Error: {0}")]
  Utf8(#[from] std::str::Utf8Error),
  /// Tauri utils, mainly extract and file move.
  #[error("Tauri API Error: {0}")]
  TauriApi(#[from] tauri_api::Error),
  /// Network error.
  #[error("Network Error: {0}")]
  Network(String),
  /// Metadata (JSON) error.
  #[error("Remote JSON Error: {0}")]
  RemoteMetadata(String),
  /// Error building updater.
  #[error("Updater Error: {0}")]
  Builder(String),
  /// Updater is not supported for current operating system or platform.
  #[error("Unsuported operating system or platform.")]
  UnsupportedPlatform,
  /// Generic updater error.
  #[error("Signature not available but pubkey provided, skipping update")]
  PubkeyButNoSignature,
  /// Triggered when there is NO error and the two versions are equals.
  /// On client side, it's important to catch this error.
  #[error("No updates available")]
  UpToDate,
}

pub type Result<T = ()> = std::result::Result<T, Error>;
