// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Runs the same IPC checks (`ui/checks.js`) under each Tauri mode on the Qt runtime:
//!
//! | mode          | pattern     | page served from                       | IPC transport          |
//! |---------------|-------------|----------------------------------------|------------------------|
//! | `brownfield`  | brownfield  | `tauri://localhost`, CSP with nonces    | `ipc://` protocol      |
//! | `isolation`   | isolation   | `tauri://localhost` + isolation iframe  | `ipc://`, via the iframe |
//! | `devserver`   | brownfield  | `devUrl` (`http://localhost:17421`)     | `ipc://` protocol      |
//! | `postmessage` | brownfield  | `tauri://localhost`, CSP without `ipc:` | `window.ipc.postMessage` fallback |
//! | `noglobal`    | brownfield  | `tauri://localhost`, no CSP, no `window.__TAURI__` | `ipc://` protocol |
//! | `remote`      | brownfield  | `http://127.0.0.1:17422` with a `remote` capability, plus a page without one (IPC must be rejected) | `ipc://` protocol |
//!
//!   QT_MODES_MODE=isolation cargo run -p qt-modes
//!   cargo test -p qt-modes          # every mode, offscreen
//!
//! Each check prints `[PASS]`/`[FAIL]`; the app exits with 0 if all passed, 1 if some failed and
//! 2 if the page never finished (e.g. IPC is broken).

use std::{
  io::{BufRead, BufReader, Write},
  net::{TcpListener, TcpStream},
  path::PathBuf,
  sync::Mutex,
  time::{Duration, Instant},
};

use serde_json::{Value, json};
use tauri::{
  AppHandle, Emitter, Listener, Manager, Runtime, State, WebviewUrl, WebviewWindowBuilder,
  ipc::{Channel, InvokeBody, Request, Response},
};

const MODES: &[&str] = &[
  "brownfield",
  "isolation",
  "devserver",
  "postmessage",
  "noglobal",
  "remote",
];
const DEV_PORT: u16 = 17421;
const REMOTE_PORT: u16 = 17422;

/// Tauri reads `tauri.conf.json` from the directory of the path given to `generate_context!`
/// (the file name is ignored), so each mode has a directory of its own.
fn context<R: Runtime>(mode: &str) -> tauri::Context<R> {
  match mode {
    // The isolation config is the crate's tauri.conf.json: tauri-build only allows the `isolation`
    // feature (needed here) when that config uses the pattern.
    "isolation" => tauri::generate_context!("tauri.conf.json"),
    "devserver" => tauri::generate_context!("modes/devserver/tauri.conf.json"),
    "postmessage" => tauri::generate_context!("modes/postmessage/tauri.conf.json"),
    "noglobal" => tauri::generate_context!("modes/noglobal/tauri.conf.json"),
    // brownfield and remote
    _ => tauri::generate_context!("modes/brownfield/tauri.conf.json"),
  }
}

// --- commands -------------------------------------------------------------------------------

#[tauri::command]
fn report(name: String, ok: bool, detail: String) {
  println!("[{}] {name}: {detail}", if ok { "PASS" } else { "FAIL" });
}

#[tauri::command]
fn done(app: AppHandle, failures: u32) {
  println!("[DONE] {failures} failure(s)");
  app.exit(if failures == 0 { 0 } else { 1 });
}

/// Echoes `value`; `isolated` is set by the isolation iframe's hook (`isolation-dist/index.html`).
#[tauri::command]
fn echo(value: Value, isolated: Option<bool>) -> Value {
  json!({ "value": value, "isolated": isolated.unwrap_or(false) })
}

/// The request body's bytes, reversed, as a raw response. Over the `ipc://` protocol the body is
/// raw; over the postMessage fallback Tauri sends binary arguments as a JSON array of numbers.
#[tauri::command]
fn echo_bytes(request: Request<'_>) -> Result<Response, String> {
  let bytes: Vec<u8> = match request.body() {
    InvokeBody::Raw(bytes) => bytes.clone(),
    InvokeBody::Json(v) => serde_json::from_value(v.clone())
      .map_err(|e| format!("expected bytes, got JSON {} ({e})", short(v)))?,
  };
  Ok(Response::new(bytes.into_iter().rev().collect::<Vec<u8>>()))
}

#[tauri::command]
fn fail() -> Result<(), String> {
  Err("expected failure".into())
}

#[tauri::command]
fn stream_numbers(count: u32, channel: Channel<u32>) -> Result<(), String> {
  for i in 0..count {
    channel.send(i).map_err(|e| e.to_string())?;
  }
  Ok(())
}

/// The title the `denied` page set after trying IPC (`denied:rejected …` when Tauri refused).
#[derive(Default)]
struct Denied(Mutex<Option<String>>);

#[tauri::command]
async fn denied_result(state: State<'_, Denied>) -> Result<String, String> {
  let deadline = Instant::now() + Duration::from_secs(15);
  loop {
    if let Some(title) = state.0.lock().unwrap().clone() {
      return Ok(title);
    }
    if Instant::now() > deadline {
      return Err("the denied page never reported".into());
    }
    std::thread::sleep(Duration::from_millis(100));
  }
}

fn short(v: &Value) -> String {
  let s = v.to_string();
  s.chars().take(80).collect()
}

// --- a static server for the devserver and remote modes ---------------------------------------

/// Serve `ui/` on `port`, on IPv4 and (if available) IPv6 loopback: Chromium may resolve
/// `localhost` to either.
fn serve(port: u16) {
  let dist = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../ui");
  let v4 = TcpListener::bind(("127.0.0.1", port))
    .unwrap_or_else(|e| panic!("qt-modes: can't listen on 127.0.0.1:{port}: {e}"));
  let listeners = [Some(v4), TcpListener::bind(("::1", port)).ok()];
  for listener in listeners.into_iter().flatten() {
    let dist = dist.clone();
    std::thread::spawn(move || {
      for stream in listener.incoming().flatten() {
        let dist = dist.clone();
        std::thread::spawn(move || {
          let _ = respond(stream, &dist);
        });
      }
    });
  }
}

fn respond(mut stream: TcpStream, dist: &std::path::Path) -> std::io::Result<()> {
  let mut reader = BufReader::new(stream.try_clone()?);
  let mut request_line = String::new();
  reader.read_line(&mut request_line)?;
  // skip the headers
  let mut line = String::new();
  while reader.read_line(&mut line)? > 2 {
    line.clear();
  }
  let path = request_line.split_whitespace().nth(1).unwrap_or("/");
  let path = path.split(['?', '#']).next().unwrap_or("/");
  let file = match path.trim_start_matches('/') {
    "" => "index.html",
    p if !p.contains("..") => p,
    _ => "",
  };
  let (status, body, content_type) = match std::fs::read(dist.join(file)) {
    Ok(body) if !file.is_empty() => {
      let ct = match file.rsplit('.').next() {
        Some("html") => "text/html; charset=utf-8",
        Some("js") => "text/javascript",
        _ => "application/octet-stream",
      };
      ("200 OK", body, ct)
    }
    _ => ("404 Not Found", b"not found".to_vec(), "text/plain"),
  };
  write!(
    stream,
    "HTTP/1.1 {status}\r\nContent-Type: {content_type}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
    body.len()
  )?;
  stream.write_all(&body)
}

// --- the app ------------------------------------------------------------------------------------

fn main() {
  env_logger::init(); // RUST_LOG=tauri_runtime_qt=debug shows every scheme request
  let mode = std::env::var("QT_MODES_MODE").unwrap_or_else(|_| "brownfield".into());
  if !MODES.contains(&mode.as_str()) {
    eprintln!("qt-modes: unknown mode {mode:?}; one of {MODES:?}");
    std::process::exit(2);
  }
  match mode.as_str() {
    "devserver" => serve(DEV_PORT),
    "remote" => serve(REMOTE_PORT),
    _ => {}
  }
  // A page whose IPC never works never calls `done`.
  std::thread::spawn(|| {
    std::thread::sleep(Duration::from_secs(120));
    println!("[TIMEOUT] the checks didn't finish");
    std::process::exit(2);
  });

  let mode_ = mode.clone();
  tauri::Builder::default()
    .runtime(tauri_runtime_qt::Qt::default())
    .manage(Denied::default())
    .invoke_handler(tauri::generate_handler![
      report,
      done,
      echo,
      echo_bytes,
      fail,
      stream_numbers,
      denied_result
    ])
    .setup(move |app| {
      let handle = app.handle().clone();
      app.listen_any("modes-ping", move |event| {
        let from: String = serde_json::from_str(event.payload()).unwrap_or_default();
        let _ = handle.emit("modes-pong", format!("pong:{from}"));
      });
      open_windows(app.handle(), &mode_)?;
      Ok(())
    })
    .run(context(&mode))
    .expect("error while running the app");
}

fn open_windows<R: Runtime>(app: &AppHandle<R>, mode: &str) -> tauri::Result<()> {
  let remote = mode == "remote";
  let (label, url, origin) = match mode {
    "devserver" => (
      "main",
      WebviewUrl::App("index.html".into()),
      format!("http://localhost:{DEV_PORT}"),
    ),
    "remote" => (
      "remote",
      WebviewUrl::External(
        format!("http://127.0.0.1:{REMOTE_PORT}/index.html")
          .parse()
          .unwrap(),
      ),
      format!("http://127.0.0.1:{REMOTE_PORT}"),
    ),
    _ => (
      "main",
      WebviewUrl::App("index.html".into()),
      "tauri://localhost".to_string(),
    ),
  };
  let config = app.config();
  let expect = json!({
    "name": mode,
    "origin": origin,
    "global": config.app.with_global_tauri,
    "isolation": mode == "isolation",
    "transport": if mode == "postmessage" { "postMessage" } else { "ipc" },
    "csp": config.app.security.csp.is_some() && !matches!(mode, "devserver" | "remote"),
    "window": label,
    "webview": label,
    "denied": remote,
  });
  WebviewWindowBuilder::new(app, label, url)
    .title(format!("qt-modes: {mode}"))
    .inner_size(700.0, 500.0)
    .initialization_script(format!("window.__QT_MODE__ = {expect};"))
    .build()?;

  if remote {
    // Not covered by capabilities/remote.json (localhost, not 127.0.0.1): IPC must be rejected.
    let app_ = app.clone();
    WebviewWindowBuilder::new(
      app,
      "denied",
      WebviewUrl::External(
        format!("http://localhost:{REMOTE_PORT}/denied.html")
          .parse()
          .unwrap(),
      ),
    )
    .title("qt-modes: denied")
    .inner_size(400.0, 200.0)
    .on_document_title_changed(move |_, title| {
      if title.starts_with("denied:") && title != "denied:pending" {
        *app_.state::<Denied>().0.lock().unwrap() = Some(title);
      }
    })
    .build()?;
  }
  Ok(())
}
