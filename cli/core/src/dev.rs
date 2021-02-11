use crate::helpers::{
  app_paths::{app_dir, tauri_dir},
  config::{get as get_config, reload as reload_config},
  manifest::rewrite_manifest,
  Logger, TauriScript,
};

use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use shared_child::SharedChild;

use std::env::{set_current_dir, set_var};
use std::ffi::OsStr;
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::{exit, Command};
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex};
use std::time::Duration;

#[derive(Default)]
pub struct Dev {
  exit_on_panic: bool,
  config: Option<String>,
}

impl Dev {
  pub fn new() -> Self {
    Default::default()
  }

  pub fn config(mut self, config: String) -> Self {
    self.config.replace(config);
    self
  }

  pub fn exit_on_panic(mut self, exit_on_panic: bool) -> Self {
    self.exit_on_panic = exit_on_panic;
    self
  }

  pub fn run(self) -> crate::Result<()> {
    let logger = Logger::new("tauri:dev");
    let tauri_path = tauri_dir();
    set_current_dir(&tauri_path)?;
    let merge_config = self.config.clone();
    let config = get_config(merge_config.as_deref())?;
    let mut _guard = None;
    let mut process: Arc<SharedChild>;

    if let Some(before_dev) = &config
      .lock()
      .unwrap()
      .as_ref()
      .unwrap()
      .build
      .before_dev_command
    {
      let mut cmd: Option<&str> = None;
      let mut args: Vec<&str> = vec![];
      for token in before_dev.split(' ') {
        if cmd.is_none() && !token.is_empty() {
          cmd = Some(token);
        } else {
          args.push(token)
        }
      }

      if let Some(cmd) = cmd {
        logger.log(format!("Running `{}`", before_dev));
        let mut command = Command::new(cmd);
        command.args(args).current_dir(app_dir()).spawn()?;
        _guard = Some(command);
      }
    }

    let dev_path = config
      .lock()
      .unwrap()
      .as_ref()
      .unwrap()
      .build
      .dev_path
      .to_string();

    let dev_path = if dev_path.starts_with("http") {
      dev_path
    } else {
      let absolute_dev_path = tauri_dir()
        .join(&config.lock().unwrap().as_ref().unwrap().build.dev_path)
        .to_string_lossy()
        .to_string();
      (*config.lock().unwrap()).as_mut().unwrap().build.dev_path = absolute_dev_path.to_string();
      absolute_dev_path
    };

    set_var("TAURI_DIR", &tauri_path);
    set_var(
      "TAURI_DIST_DIR",
      tauri_path.join(&config.lock().unwrap().as_ref().unwrap().build.dist_dir),
    );
    set_var(
      "TAURI_CONFIG",
      serde_json::to_string(&*config.lock().unwrap())?,
    );

    rewrite_manifest(config.clone())?;

    // __tauri.js
    {
      let config_guard = config.lock().unwrap();
      let config_ = config_guard.as_ref().unwrap();
      let tauri_script = TauriScript::new()
        .global_tauri(config_.build.with_global_tauri)
        .get();
      let tauri_script_path = PathBuf::from(&config_.build.dist_dir).join("__tauri.js");
      let mut tauri_script_file = File::create(tauri_script_path)?;
      tauri_script_file.write_all(tauri_script.as_bytes())?;
    }

    let (child_wait_tx, child_wait_rx) = channel();
    let child_wait_rx = Arc::new(Mutex::new(child_wait_rx));

    process = self.start_app(child_wait_rx.clone());

    let (tx, rx) = channel();

    let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();
    watcher.watch(tauri_path.join("src"), RecursiveMode::Recursive)?;
    watcher.watch(tauri_path.join("Cargo.toml"), RecursiveMode::Recursive)?;
    watcher.watch(tauri_path.join("tauri.conf.json"), RecursiveMode::Recursive)?;
    if !dev_path.starts_with("http") {
      watcher.watch(
        config
          .lock()
          .unwrap()
          .as_ref()
          .unwrap()
          .build
          .dev_path
          .to_string(),
        RecursiveMode::Recursive,
      )?;
    }

    loop {
      if let Ok(event) = rx.recv() {
        let event_path = match event {
          DebouncedEvent::Create(path) => Some(path),
          DebouncedEvent::Remove(path) => Some(path),
          DebouncedEvent::Rename(_, dest) => Some(dest),
          DebouncedEvent::Write(path) => Some(path),
          _ => None,
        };

        if let Some(event_path) = event_path {
          if event_path.file_name() == Some(OsStr::new("tauri.conf.json")) {
            reload_config(merge_config.as_deref())?;
            (*config.lock().unwrap()).as_mut().unwrap().build.dev_path = dev_path.to_string();
            rewrite_manifest(config.clone())?;
            set_var("TAURI_CONFIG", serde_json::to_string(&*config)?);
          } else {
            // When tauri.conf.json is changed, rewrite_manifest will be called
            // which will trigger the watcher again
            // So the app should only be started when a file other than tauri.conf.json is changed
            let _ = child_wait_tx.send(());
            process.kill()?;
            process = self.start_app(child_wait_rx.clone());
          }
        }
      }
    }
  }

  fn start_app(&self, child_wait_rx: Arc<Mutex<Receiver<()>>>) -> Arc<SharedChild> {
    let mut command = Command::new("cargo");
    command.arg("run");
    let child = SharedChild::spawn(&mut command).expect("failed to run cargo");
    let child_arc = Arc::new(child);

    let child_clone = child_arc.clone();
    let exit_on_panic = self.exit_on_panic;
    std::thread::spawn(move || {
      let status = child_clone.wait().expect("failed to wait on child");
      if exit_on_panic {
        // we exit if the status is a success code (app closed) or code is 101 (compilation error)
        // if the process wasn't killed by the file watcher
        if (status.success() || status.code() == Some(101))
          // `child_wait_rx` indicates that the process was killed by the file watcher
          && child_wait_rx
          .lock()
          .expect("failed to get child_wait_rx lock")
          .try_recv()
          .is_err()
        {
          exit(0);
        }
      } else if status.success() {
        // if we're no exiting on panic, we only exit if the status is a success code (app closed)
        exit(0);
      }
    });

    child_arc
  }
}
