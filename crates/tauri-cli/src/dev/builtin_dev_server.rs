// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use axum::{
  extract::{ws, State, WebSocketUpgrade},
  http::{header, StatusCode, Uri},
  response::{IntoResponse, Response},
};
use html5ever::{namespace_url, ns, LocalName, QualName};
use kuchiki::{traits::TendrilSink, NodeRef};
use std::{
  net::{IpAddr, SocketAddr},
  path::{Path, PathBuf},
  thread,
  time::Duration,
};
use tauri_utils::mime_type::MimeType;
use tokio::sync::broadcast::{channel, Sender};

const RELOAD_SCRIPT: &str = include_str!("./auto-reload.js");

#[derive(Clone)]
struct ServerState {
  dir: PathBuf,
  address: SocketAddr,
  tx: Sender<()>,
}

pub fn start<P: AsRef<Path>>(dir: P, ip: IpAddr, port: Option<u16>) -> crate::Result<SocketAddr> {
  let dir = dir.as_ref();
  let dir = dunce::canonicalize(dir)?;

  // bind port and tcp listener
  let auto_port = port.is_none();
  let mut port = port.unwrap_or(1430);
  let (tcp_listener, address) = loop {
    let address = SocketAddr::new(ip, port);
    if let Ok(tcp) = std::net::TcpListener::bind(address) {
      tcp.set_nonblocking(true)?;
      break (tcp, address);
    }

    if !auto_port {
      anyhow::bail!("Couldn't bind to {port} on {ip}");
    }

    port += 1;
  };

  let (tx, _) = channel(1);

  // watch dir for changes
  let tx_c = tx.clone();
  watch(dir.clone(), move || {
    let _ = tx_c.send(());
  });

  let state = ServerState { dir, tx, address };

  // start router thread
  std::thread::spawn(move || {
    tokio::runtime::Builder::new_current_thread()
      .enable_io()
      .build()
      .expect("failed to start tokio runtime for builtin dev server")
      .block_on(async move {
        let router = axum::Router::new()
          .fallback(handler)
          .route("/__tauri_cli", axum::routing::get(ws_handler))
          .with_state(state);

        axum::serve(tokio::net::TcpListener::from_std(tcp_listener)?, router).await
      })
      .expect("builtin server errored");
  });

  Ok(address)
}

async fn handler(uri: Uri, state: State<ServerState>) -> impl IntoResponse {
  // Frontend files should not contain query parameters. This seems to be how vite handles it.
  let uri = uri.path();

  let uri = if uri == "/" {
    uri
  } else {
    uri.strip_prefix('/').unwrap_or(uri)
  };

  let bytes = fs_read_scoped(state.dir.join(uri), &state.dir)
    .or_else(|_| fs_read_scoped(state.dir.join(format!("{}.html", &uri)), &state.dir))
    .or_else(|_| fs_read_scoped(state.dir.join(format!("{}/index.html", &uri)), &state.dir))
    .or_else(|_| std::fs::read(state.dir.join("index.html")));

  match bytes {
    Ok(mut bytes) => {
      let mime_type = MimeType::parse_with_fallback(&bytes, uri, MimeType::OctetStream);
      if mime_type == MimeType::Html.to_string() {
        bytes = inject_address(bytes, &state.address);
      }
      (StatusCode::OK, [(header::CONTENT_TYPE, mime_type)], bytes)
    }
    Err(_) => (
      StatusCode::NOT_FOUND,
      [(header::CONTENT_TYPE, "text/plain".into())],
      vec![],
    ),
  }
}

async fn ws_handler(ws: WebSocketUpgrade, state: State<ServerState>) -> Response {
  ws.on_upgrade(move |mut ws| async move {
    let mut rx = state.tx.subscribe();
    while tokio::select! {
        _ = ws.recv() => return,
        fs_reload_event = rx.recv() => fs_reload_event.is_ok(),
    } {
      let msg = ws::Message::Text(r#"{"reload": true}"#.to_owned());
      if ws.send(msg).await.is_err() {
        break;
      }
    }
  })
}

fn inject_address(html_bytes: Vec<u8>, address: &SocketAddr) -> Vec<u8> {
  fn with_html_head<F: FnOnce(&NodeRef)>(document: &mut NodeRef, f: F) {
    if let Ok(ref node) = document.select_first("head") {
      f(node.as_node())
    } else {
      let node = NodeRef::new_element(
        QualName::new(None, ns!(html), LocalName::from("head")),
        None,
      );
      f(&node);
      document.prepend(node)
    }
  }

  let mut document = kuchiki::parse_html().one(String::from_utf8_lossy(&html_bytes).into_owned());
  with_html_head(&mut document, |head| {
    let script = RELOAD_SCRIPT.replace("{{reload_url}}", &format!("ws://{address}/__tauri_cli"));
    let script_el = NodeRef::new_element(QualName::new(None, ns!(html), "script".into()), None);
    script_el.append(NodeRef::new_text(script));
    head.prepend(script_el);
  });

  tauri_utils::html::serialize_node(&document)
}

fn fs_read_scoped(path: PathBuf, scope: &Path) -> crate::Result<Vec<u8>> {
  let path = dunce::canonicalize(path)?;
  if path.starts_with(scope) {
    std::fs::read(path).map_err(Into::into)
  } else {
    anyhow::bail!("forbidden path")
  }
}

fn watch<F: Fn() + Send + 'static>(dir: PathBuf, handler: F) {
  thread::spawn(move || {
    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher = notify_debouncer_mini::new_debouncer(Duration::from_secs(1), tx)
      .expect("failed to start builtin server fs watcher");

    watcher
      .watcher()
      .watch(&dir, notify::RecursiveMode::Recursive)
      .expect("builtin server failed to watch dir");

    loop {
      if rx.recv().is_ok() {
        handler();
      }
    }
  });
}
