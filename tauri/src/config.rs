use std::env;

#[derive(Deserialize, Clone)]
#[serde(tag = "window", rename_all = "camelCase")]
pub struct WindowConfig {
  #[serde(default = "default_width")]
  pub width: i32,
  #[serde(default = "default_height")]
  pub height: i32,
  #[serde(default = "default_resizable")]
  pub resizable: bool,
  #[serde(default = "default_title")]
  pub title: String,
}

fn default_width() -> i32 {
  800
}

fn default_height() -> i32 {
  600
}

fn default_resizable() -> bool {
  true
}

fn default_title() -> String {
  "Tauri App".to_string()
}

fn default_window() -> WindowConfig {
  return WindowConfig {
    width: default_width(),
    height: default_height(),
    resizable: default_resizable(),
    title: default_title(),
  };
}

#[derive(Deserialize, Clone)]
#[serde(tag = "embeddedServer", rename_all = "camelCase")]
pub struct EmbeddedServerConfig {
  #[serde(default = "default_host")]
  pub host: String,
  #[serde(default = "default_port")]
  pub port: String,
}

fn default_host() -> String {
  "http://127.0.0.1".to_string()
}

fn default_port() -> String {
  "random".to_string()
}

fn default_embedded_server() -> EmbeddedServerConfig {
  EmbeddedServerConfig {
    host: default_host(),
    port: default_port(),
  }
}

#[derive(Deserialize, Clone)]
#[serde(tag = "tauri", rename_all = "camelCase")]
pub struct TauriConfig {
  #[serde(default = "default_window")]
  pub window: WindowConfig,
  #[serde(default = "default_embedded_server")]
  pub embedded_server: EmbeddedServerConfig,
}

#[derive(Deserialize, Clone)]
#[serde(tag = "build", rename_all = "camelCase")]
pub struct BuildConfig {
  #[serde(default = "default_dev_path")]
  pub dev_path: String,
}

fn default_dev_path() -> String {
  "".to_string()
}

#[derive(Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Config {
  #[serde(default = "default_tauri")]
  pub tauri: TauriConfig,
  #[serde(default = "default_build")]
  pub build: BuildConfig,
}

fn default_tauri() -> TauriConfig {
  TauriConfig {
    window: default_window(),
    embedded_server: default_embedded_server(),
  }
}

fn default_build() -> BuildConfig {
  BuildConfig {
    dev_path: default_dev_path(),
  }
}

pub fn get() -> Config {
  match option_env!("TAURI_CONFIG") {
    Some(config) => serde_json::from_str(config)
      .expect("failed to parse TAURI_CONFIG env"),
    None => serde_json::from_str(include_str!(concat!(env!("TAURI_DIR"), "/tauri.conf.json")))
      .expect("failed to read tauri.conf.json")
  }
}
