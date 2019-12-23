use std::env;
use std::fs;
use std::path::Path;

#[derive(Deserialize)]
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

#[derive(Deserialize)]
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

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Config {
  #[serde(default = "default_window")]
  pub window: WindowConfig,
  #[serde(default = "default_embedded_server")]
  pub embedded_server: EmbeddedServerConfig,
  #[serde(default = "default_dev_path")]
  pub dev_path: String,
  #[serde(default = "default_inlined_assets")]
  pub inlined_assets: Vec<String>,
}

fn default_inlined_assets() -> Vec<String> {
  Vec::new()
}

fn default_dev_path() -> String {
  "".to_string()
}

pub fn get() -> Config {
  let path_str = concat!(env!("TAURI_DIR"), "/config.json");
  let path = Path::new(&path_str);
  let config_file: String;
  if path.is_relative() {
      config_file = fs::read_to_string(Path::new(&env!("PWD")).join(path).as_path()).unwrap();
  } else {
      config_file = fs::read_to_string(path_str).unwrap();
  }
  serde_json::from_str(&config_file).unwrap()
}
