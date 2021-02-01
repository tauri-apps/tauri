use crate::helpers::{
  app_paths::{app_dir, tauri_dir},
  config::{get as get_config, reload as reload_config, ConfigHandle},
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

    let running_dev_server = config
      .lock()
      .unwrap()
      .as_ref()
      .unwrap()
      .build
      .dev_path
      .starts_with("http");

    let new_dev_path = if running_dev_server {
      let dev_path = Url::parse(&config.lock().unwrap().as_ref().unwrap().build.dev_path)?;
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
      let config_ = config.clone();
      std::thread::spawn(move || proxy_dev_server(config_, &proxy_path, proxy_port));

      format!(
        "http://{}:{}",
        dev_path.host_str().expect("failed to read dev_path host"),
        proxy_port
      )
    } else {
      tauri_dir()
        .join(&config.lock().unwrap().as_ref().unwrap().build.dev_path)
        .to_string_lossy()
        .to_string()
    };

    (*config.lock().unwrap()).as_mut().unwrap().build.dev_path = new_dev_path.to_string();

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

    let (child_wait_tx, child_wait_rx) = channel();
    let child_wait_rx = Arc::new(Mutex::new(child_wait_rx));

    process = self.start_app(child_wait_rx.clone());

    let (tx, rx) = channel();

    let mut watcher = watcher(tx, Duration::from_secs(1)).unwrap();
    watcher.watch(tauri_path.join("src"), RecursiveMode::Recursive)?;
    watcher.watch(tauri_path.join("Cargo.toml"), RecursiveMode::Recursive)?;
    watcher.watch(tauri_path.join("tauri.conf.json"), RecursiveMode::Recursive)?;
    if !running_dev_server {
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
            (*config.lock().unwrap()).as_mut().unwrap().build.dev_path = new_dev_path.to_string();
            rewrite_manifest(config.clone())?;
            set_var("TAURI_CONFIG", serde_json::to_string(&*config)?);
          } else {
            // When tauri.conf.json is changed, rewrite_manifest will be called
            // which will trigger the watcher again
            // So the app should only be started when a file other than tauri.conf.json is changed
            let _ = child_wait_tx.send(true);
            process.kill()?;
            process = self.start_app(child_wait_rx.clone());
          }
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

fn proxy_dev_server(config: ConfigHandle, dev_path: &Url, dev_port: u16) -> crate::Result<()> {
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
      let config_guard = config.lock().unwrap();
      let config = config_guard.as_ref().unwrap();
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
