use serde::Deserialize;

use std::{fs, path};

#[derive(PartialEq, Deserialize, Clone, Debug)]
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
  #[serde(default)]
  pub fullscreen: bool,
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
  WindowConfig {
    width: default_width(),
    height: default_height(),
    resizable: default_resizable(),
    title: default_title(),
    fullscreen: false,
  }
}

#[derive(PartialEq, Deserialize, Clone, Debug)]
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

#[derive(PartialEq, Deserialize, Clone, Debug)]
#[serde(tag = "tauri", rename_all = "camelCase")]
pub struct TauriConfig {
  #[serde(default = "default_window")]
  pub window: WindowConfig,
  #[serde(default = "default_embedded_server")]
  pub embedded_server: EmbeddedServerConfig,
}

#[derive(PartialEq, Deserialize, Clone, Debug)]
#[serde(tag = "build", rename_all = "camelCase")]
pub struct BuildConfig {
  #[serde(default = "default_dev_path")]
  pub dev_path: String,
}

fn default_dev_path() -> String {
  "".to_string()
}

#[derive(PartialEq, Deserialize, Clone, Debug)]
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

pub fn get() -> crate::Result<Config> {
  match option_env!("TAURI_CONFIG") {
    Some(config) => Ok(serde_json::from_str(config).expect("failed to parse TAURI_CONFIG env")),
    None => {
      let env_var = envmnt::get_or("TAURI_DIR", "../dist");
      let path = path::Path::new(&env_var);
      let contents = fs::read_to_string(path.join("tauri.conf.json"))?;

      Ok(serde_json::from_str(&contents).expect("failed to read tauri.conf.json"))
    }
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
          title: String::from("Tauri App"),
          fullscreen: false,
        },
        embedded_server: EmbeddedServerConfig {
          host: String::from("http://127.0.0.1"),
          port: String::from("random"),
        },
      },
      build: BuildConfig {
        dev_path: String::from("../dist"),
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
      Ok(c) => {
        println!("{:?}", c);
        assert_eq!(c, test_config)
      }
      Err(_) => assert!(false),
    }
  }

  #[test]
  // test all of the default functions
  fn test_defaults() {
    // get default tauri config
    let t_config = default_tauri();
    // get default build config
    let b_config = default_build();
    // get default dev path
    let d_path = default_dev_path();
    // get default embedded server
    let de_server = default_embedded_server();
    // get default window
    let d_window = default_window();
    // get default title
    let d_title = default_title();

    // create a tauri config.
    let tauri = TauriConfig {
      window: WindowConfig {
        width: 800,
        height: 600,
        resizable: true,
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
    assert_eq!(d_path, String::from(""));
    assert_eq!(d_title, tauri.window.title);
    assert_eq!(d_window, tauri.window);
  }
}
