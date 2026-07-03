// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use axum::{
  extract::{ws, State, WebSocketUpgrade},
  http::{header, StatusCode, Uri},
  response::{IntoResponse, Response},
};
use std::{
  net::{IpAddr, SocketAddr},
  path::{Path, PathBuf},
  time::Duration,
};
use tauri_utils::mime_type::MimeType;
use tokio::sync::broadcast::{channel, Sender};

use crate::error::ErrorExt;

const RELOAD_SCRIPT: &str = include_str!("./auto-reload.js");

#[derive(Clone)]
struct ServerState {
  dir: PathBuf,
  address: SocketAddr,
  tx: Sender<()>,
}

pub async fn start<P: AsRef<Path>>(
  dir: P,
  ip: IpAddr,
  port: Option<u16>,
) -> crate::Result<SocketAddr> {
  let dir = dir.as_ref();
  let dir =
    dunce::canonicalize(dir).fs_context("failed to canonicalize path", dir.to_path_buf())?;

  // bind port and tcp listener
  let auto_port = port.is_none();
  let mut port = port.unwrap_or(1430);
  let (tcp_listener, address) = loop {
    let address = SocketAddr::new(ip, port);
    if let Ok(tcp) = tokio::net::TcpListener::bind(address).await {
      break (tcp, address);
    }

    if !auto_port {
      crate::error::bail!("Couldn't bind to {port} on {ip}");
    }

    port += 1;
  };

  let (tx, _) = channel(1);

  // watch dir for changes
  let tx_c = tx.clone();
  let watcher = watch(&dir, move || {
    let _ = tx_c.send(());
  });

  let state = ServerState { dir, tx, address };

  // the server task lives as long as the runtime; when the runtime shuts
  // down the task is dropped, closing the listener and the fs watcher with it
  tokio::spawn(async move {
    // keep the fs watcher alive for as long as the server is running
    let _watcher = watcher;

    let router = axum::Router::new()
      .fallback(handler)
      .route("/__tauri_cli", axum::routing::get(ws_handler))
      .with_state(state);

    axum::serve(tcp_listener, router)
      .await
      .expect("builtin server errored");
  });

  Ok(address)
}

async fn handler(uri: Uri, state: State<ServerState>) -> impl IntoResponse {
  // Frontend files should not contain query parameters. This seems to be how Vite handles it.
  let uri = uri.path();

  let uri = if uri == "/" {
    uri
  } else {
    uri.strip_prefix('/').unwrap_or(uri)
  };

  let mut bytes = fs_read_scoped(state.dir.join(uri), &state.dir).await;
  if bytes.is_err() {
    bytes = fs_read_scoped(state.dir.join(format!("{}.html", &uri)), &state.dir).await;
  }
  if bytes.is_err() {
    bytes = fs_read_scoped(state.dir.join(format!("{}/index.html", &uri)), &state.dir).await;
  }
  let bytes = match bytes {
    Ok(bytes) => Ok(bytes),
    Err(_) => tokio::fs::read(state.dir.join("index.html")).await,
  };

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
      let msg = ws::Message::Text(r#"{"reload": true}"#.into());
      if ws.send(msg).await.is_err() {
        break;
      }
    }
  })
}

fn inject_address(html_bytes: Vec<u8>, address: &SocketAddr) -> Vec<u8> {
  let document = tauri_utils::html2::parse_doc(String::from_utf8_lossy(&html_bytes).into_owned());

  tauri_utils::html2::append_script_to_head(
    &document,
    &RELOAD_SCRIPT.replace("{{reload_url}}", &format!("ws://{address}/__tauri_cli")),
  );

  tauri_utils::html2::serialize_doc(&document)
}

async fn fs_read_scoped(path: PathBuf, scope: &Path) -> crate::Result<Vec<u8>> {
  let path = tokio::fs::canonicalize(&path)
    .await
    .fs_context("failed to canonicalize path", path)?;
  // simplify UNC paths on Windows so they match the dunce-canonicalized scope
  if dunce::simplified(&path).starts_with(scope) {
    tokio::fs::read(&path)
      .await
      .fs_context("failed to read file", &path)
  } else {
    crate::error::bail!("forbidden path")
  }
}

fn watch<F: Fn() + Send + 'static>(
  dir: &Path,
  handler: F,
) -> notify_debouncer_full::Debouncer<
  notify::RecommendedWatcher,
  notify_debouncer_full::RecommendedCache,
> {
  // the handler is called directly on the debouncer's worker thread,
  // it only signals a tokio broadcast channel so it never blocks
  let mut watcher = notify_debouncer_full::new_debouncer(
    Duration::from_secs(1),
    None,
    move |r: notify_debouncer_full::DebounceEventResult| {
      if let Ok(events) = r {
        if let Some(event) = events.first() {
          if !event.kind.is_access() {
            handler();
          }
        }
      }
    },
  )
  .expect("failed to start builtin server fs watcher");

  watcher
    .watch(dir, notify::RecursiveMode::Recursive)
    .expect("builtin server failed to watch dir");

  watcher
}
