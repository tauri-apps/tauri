use serde::Deserialize;

use std::env;

#[derive(PartialEq, Deserialize, Clone, Debug)]
#[serde(default, tag = "window", rename_all = "camelCase")]
pub struct WindowConfig {
  pub width: i32,
  pub height: i32,
  pub resizable: bool,
  pub frameless: bool,
  pub fullscreen: bool,
  pub title: String,
}

impl Default for WindowConfig {
  fn default() -> Self {
    Self {
      width: 800,
      height: 600,
      resizable: true,
      frameless: false,
      fullscreen: false,
      title: "Tauri App".to_string(),
    }
  }
}

#[derive(PartialEq, Deserialize, Clone, Debug)]
#[serde(default, tag = "embeddedServer", rename_all = "camelCase")]
pub struct EmbeddedServerConfig {
  pub host: String,
  pub port: String,
}

impl Default for EmbeddedServerConfig {
  fn default() -> Self {
    Self {
      host: "http://127.0.0.1".to_string(),
      port: "random".to_string(),
    }
  }
}

#[derive(PartialEq, Default, Deserialize, Clone, Debug)]
#[serde(default, tag = "tauri", rename_all = "camelCase")]
pub struct TauriConfig {
  pub window: WindowConfig,
  pub embedded_server: EmbeddedServerConfig,
}

#[derive(PartialEq, Default, Deserialize, Clone, Debug)]
#[serde(default, tag = "build", rename_all = "camelCase")]
pub struct BuildConfig {
  pub dev_path: String,
}

#[derive(PartialEq, Default, Deserialize, Clone, Debug)]
#[serde(default, rename_all = "camelCase")]
pub struct Config {
  pub tauri: TauriConfig,
  pub build: BuildConfig,
}

pub fn get() -> crate::Result<Config> {
  match option_env!("TAURI_CONFIG") {
    Some(config) => Ok(serde_json::from_str(config).expect("failed to parse TAURI_CONFIG env")),
    None => Ok(
      serde_json::from_str(include_str!(concat!(env!("TAURI_DIR"), "/tauri.conf.json")))
        .expect("failed to read tauri.conf.json"),
    ),
  }
}

#[cfg(test)]
mod test {
  use super::*;
  // generate a test_config based on the test fixture
  fn create_test_config() -> Config {
    Config {
      tauri: TauriConfig {
        window: WindowConfig {
          width: 800,
          height: 600,
          resizable: true,
          frameless: false,
          title: String::from("Tauri App"),
          fullscreen: false,
        },
        embedded_server: EmbeddedServerConfig {
          host: String::from("http://127.0.0.1"),
          port: String::from("random"),
        },
      },
      build: BuildConfig {
        dev_path: String::from("http://localhost:4000"),
      },
    }
  }

  #[test]
  // test the get function.  Will only resolve to true if the TAURI_CONFIG variable is set properly to the fixture.
  fn test_get() {
    // get test_config
    let test_config = create_test_config();

    // call get();
    let config = get();

    // check to see if there is an OK or Err, on Err fail test.
    match config {
      // On Ok, check that the config is the same as the test config.
      Ok(c) => assert_eq!(c, test_config),
      Err(_) => assert!(false),
    }
  }

  #[test]
  // test all of the default functions
  fn test_defaults() {
    // get default tauri config
    let t_config = TauriConfig::default();
    // get default build config
    let b_config = BuildConfig::default();
    // get default dev path
    let ref d_path = b_config.dev_path;
    // get default embedded server
    let de_server = EmbeddedServerConfig::default();
    // get default window
    let d_window = WindowConfig::default();
    // get default title
    let ref d_title = d_window.title;

    // create a tauri config.
    let tauri = TauriConfig {
      window: WindowConfig {
        width: 800,
        height: 600,
        resizable: true,
        frameless: false,
        title: String::from("Tauri App"),
        fullscreen: false,
      },
      embedded_server: EmbeddedServerConfig {
        host: String::from("http://127.0.0.1"),
        port: String::from("random"),
      },
    };

    // create a build config
    let build = BuildConfig {
      dev_path: String::from(""),
    };

    // test the configs
    assert_eq!(t_config, tauri);
    assert_eq!(b_config, build);
    assert_eq!(de_server, tauri.embedded_server);
    assert_eq!(d_path, "");
    assert_eq!(d_title, &tauri.window.title);
    assert_eq!(d_window, tauri.window);
  }
}
