use crate::helpers::{
  app_paths::{app_dir, tauri_dir},
  config::{get as get_config, reload as reload_config},
  manifest::rewrite_manifest,
  Logger, TauriHtml,
};

use attohttpc::{Method, RequestBuilder};
use http::header::HeaderName;
use notify::{watcher, DebouncedEvent, RecursiveMode, Watcher};
use shared_child::SharedChild;
use tiny_http::{Response, Server};
use url::Url;

use std::env::set_var;
use std::ffi::OsStr;
use std::process::{exit, Child, Command};
use std::sync::mpsc::{channel, Receiver};
use std::sync::{Arc, Mutex};
use std::thread::sleep;
use std::time::Duration;

struct ChildGuard(Child);

impl Drop for ChildGuard {
  fn drop(&mut self) {
    let _ = self.0.kill();
  }
}

#[derive(Default)]
pub struct Dev {
  exit_on_panic: bool,
}

impl Dev {
  pub fn new() -> Self {
    Default::default()
  }

  pub fn exit_on_panic(mut self, exit_on_panic: bool) -> Self {
    self.exit_on_panic = exit_on_panic;
    self
  }

  pub fn run(self) -> crate::Result<()> {
    let logger = Logger::new("tauri:dev");
    let tauri_path = tauri_dir();
    let config = get_config()?;
    let mut config_mut = config.clone();
    let mut _guard = None;
    let mut process: Arc<SharedChild>;
    let new_dev_path: String;

    if let Some(before_dev) = &config.build.before_dev_command {
      let mut cmd: Option<&str> = None;
      let mut args: Vec<&str> = vec![];
      for token in before_dev.split(' ') {
        if cmd.is_none() {
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

    let running_dev_server = config.build.dev_path.starts_with("http");

    if running_dev_server {
      let dev_path = Url::parse(&config.build.dev_path)?;
      let dev_port = dev_path.port().unwrap_or(80);

      let timeout = Duration::from_secs(3);
      let wait_time = Duration::from_secs(30);
      let mut total_time = timeout;
      while RequestBuilder::new(Method::GET, &dev_path).send().is_err() {
        logger.warn("Waiting for your dev server to start...");
        sleep(timeout);
        total_time += timeout;
        if total_time == wait_time {
          logger.error(format!(
            "Couldn't connect to {} after {}s. Please make sure that's the URL to your dev server.",
            dev_path,
            total_time.as_secs()
          ));
          exit(1);
        }
      }

      let proxy_path = dev_path.clone();
      let proxy_port = dev_port + 1;

      logger.log(format!("starting dev proxy on port {}", proxy_port));
      std::thread::spawn(move || proxy_dev_server(&proxy_path, proxy_port));

      new_dev_path = format!(
        "http://{}:{}",
        dev_path.host_str().expect("failed to read dev_path host"),
        proxy_port
      );
    } else {
      new_dev_path = tauri_dir()
        .join(&config.build.dev_path)
        .to_string_lossy()
        .to_string();
    }

    config_mut.build.dev_path = new_dev_path.clone();

    set_var("TAURI_DIR", &tauri_path);
    set_var("TAURI_DIST_DIR", tauri_path.join(&config.build.dist_dir));
    set_var("TAURI_CONFIG", serde_json::to_string(&config_mut)?);

    rewrite_manifest()?;

    let (child_wait_tx, child_wait_rx) = channel();
    let child_wait_rx = Arc::new(Mutex::new(child_wait_rx));

    process = self.start_app(child_wait_rx.clone());

    let (tx, rx) = channel();

    let mut watcher = watcher(tx, Duration::from_secs(2)).unwrap();
    watcher.watch(tauri_path.join("src"), RecursiveMode::Recursive)?;
    watcher.watch(tauri_path.join("Cargo.toml"), RecursiveMode::Recursive)?;
    watcher.watch(tauri_path.join("tauri.conf.json"), RecursiveMode::Recursive)?;
    if !running_dev_server {
      watcher.watch(config_mut.build.dev_path, RecursiveMode::Recursive)?;
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
          let _ = child_wait_tx.send(true);
          process.kill()?;
          if event_path.file_name() == Some(OsStr::new("tauri.conf.json")) {
            config_mut = reload_config()?.clone();
            rewrite_manifest()?;
            config_mut.build.dev_path = new_dev_path.clone();
            set_var("TAURI_CONFIG", serde_json::to_string(&config_mut)?);
          }

          process = self.start_app(child_wait_rx.clone());
        }
      }
    }
  }

  fn start_app(&self, child_wait_rx: Arc<Mutex<Receiver<bool>>>) -> Arc<SharedChild> {
    let mut command = Command::new("cargo");
    command.arg("run").current_dir(tauri_dir());
    let child = SharedChild::spawn(&mut command).expect("failed to run cargo");
    let child_arc = Arc::new(child);

    if self.exit_on_panic {
      let child_clone = child_arc.clone();
      std::thread::spawn(move || {
        child_clone.wait().expect("failed to wait on child");
        if child_wait_rx
          .lock()
          .expect("failed to get child_wait_rx lock")
          .try_recv()
          .is_err()
        {
          std::process::exit(1);
        }
      });
    }

    child_arc
  }
}

fn proxy_dev_server(dev_path: &Url, dev_port: u16) -> crate::Result<()> {
  let config = get_config()?;

  let server_url = format!(
    "{}:{}",
    dev_path.host_str().expect("failed to read dev_path host"),
    dev_port,
  );
  let server = Server::http(server_url).expect("failed to create proxy server");
  for request in server.incoming_requests() {
    let request_url = request.url();
    let mut request_builder = RequestBuilder::new(
      Method::from_bytes(request.method().to_string().as_bytes()).unwrap(),
      dev_path.join(&request_url)?.to_string(),
    );

    for header in request.headers() {
      request_builder = request_builder.header(
        HeaderName::from_bytes(header.field.as_str().as_bytes())?,
        header.value.as_str(),
      );
    }

    if request_url == "/" {
      let response = request_builder.send()?.text()?;
      let tauri_html = TauriHtml::new(&config.build.dist_dir, response)
        .global_tauri(config.build.with_global_tauri)
        .generate()?;
      request.respond(Response::from_data(tauri_html))?;
    } else {
      let response = request_builder.send()?.bytes()?;
      request.respond(Response::from_data(response))?;
    }
  }
  Ok(())
}
