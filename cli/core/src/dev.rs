use crate::helpers::{
  app_paths::{app_dir, tauri_dir},
  config::{get as get_config, reload as reload_config},
  manifest::rewrite_manifest,
  Logger,
};

use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use once_cell::sync::OnceCell;
use shared_child::SharedChild;

use std::{
  env::set_current_dir,
  ffi::OsStr,
  process::{exit, Child, Command},
  sync::{
    mpsc::{channel, Receiver},
    Arc, Mutex,
  },
  time::Duration,
};

static BEFORE_DEV: OnceCell<Mutex<Child>> = OnceCell::new();

fn kill_before_dev_process() {
  if let Some(child) = BEFORE_DEV.get() {
    let _ = child.lock().unwrap().kill();
  }
}

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
    let mut process: Arc<SharedChild>;

    if let Some(before_dev) = &config
      .lock()
      .unwrap()
      .as_ref()
      .unwrap()
      .build
      .before_dev_command
    {
      if !before_dev.is_empty() {
        logger.log(format!("Running `{}`", before_dev));
        #[cfg(target_os = "windows")]
        let child = Command::new("cmd")
          .arg("/C")
          .arg(before_dev)
          .current_dir(app_dir())
          .spawn()?;
        #[cfg(not(target_os = "windows"))]
        let child = Command::new("sh")
          .arg("-c")
          .arg(before_dev)
          .current_dir(app_dir())
          .spawn()?;
        BEFORE_DEV.set(Mutex::new(child)).unwrap();
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

    rewrite_manifest(config.clone())?;

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
            rewrite_manifest(config.clone())?;
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
    command.args(&["run", "--no-default-features"]);
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
          kill_before_dev_process();
          exit(0);
        }
      } else if status.success() {
        // if we're no exiting on panic, we only exit if the status is a success code (app closed)
        kill_before_dev_process();
        exit(0);
      }
    });

    child_arc
  }
}
