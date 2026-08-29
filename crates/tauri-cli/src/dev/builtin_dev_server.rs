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
  thread,
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

pub fn start<P: AsRef<Path>>(dir: P, ip: IpAddr, port: Option<u16>) -> crate::Result<SocketAddr> {
  let dir = dir.as_ref();
  let dir =
    dunce::canonicalize(dir).fs_context("failed to canonicalize path", dir.to_path_buf())?;

  // bind port and tcp listener
  let auto_port = port.is_none();
  let mut port = port.unwrap_or(1430);
  let (tcp_listener, address) = loop {
    let address = SocketAddr::new(ip, port);
    if let Ok(tcp) = std::net::TcpListener::bind(address) {
      tcp.set_nonblocking(true).unwrap();
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
  // Frontend files should not contain query parameters. This seems to be how Vite handles it.
  let uri = uri.path();

  let uri = if uri == "/" {
    uri
  } else {
    uri.strip_prefix('/').unwrap_or(uri)
  };

  match resolve_asset(&state.dir, uri) {
    Some(mut bytes) => {
      let mime_type = MimeType::parse_with_fallback(&bytes, uri, MimeType::OctetStream);
      if mime_type == MimeType::Html.to_string() {
        bytes = inject_address(bytes, &state.address);
      }
      (StatusCode::OK, [(header::CONTENT_TYPE, mime_type)], bytes)
    }
    None => (
      StatusCode::NOT_FOUND,
      [(header::CONTENT_TYPE, "text/plain".into())],
      format!("asset not found: /{}", uri.trim_start_matches('/')).into_bytes(),
    ),
  }
}

/// Resolves a request path against the dist directory, mirroring the fallback
/// chain of the `tauri://` protocol: exact path, `{path}.html`,
/// `{path}/index.html`, then the SPA `index.html` fallback.
/// Paths with a static subresource extension (`.js`, `.css`, images, ...) do
/// not fall back, so a missing file is answered with a 404 instead of an HTML
/// document.
fn resolve_asset(dir: &Path, uri: &str) -> Option<Vec<u8>> {
  let exact = fs_read_scoped(dir.join(uri), dir).ok();

  if tauri_utils::mime_type::has_subresource_extension(uri) {
    return exact;
  }

  exact
    .or_else(|| fs_read_scoped(dir.join(format!("{uri}.html")), dir).ok())
    .or_else(|| fs_read_scoped(dir.join(format!("{uri}/index.html")), dir).ok())
    .or_else(|| {
      let bytes = std::fs::read(dir.join("index.html")).ok();
      if bytes.is_some() {
        log::warn!("asset `/{uri}` not found; serving `index.html` instead (SPA fallback)");
      }
      bytes
    })
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

fn fs_read_scoped(path: PathBuf, scope: &Path) -> crate::Result<Vec<u8>> {
  let path = dunce::canonicalize(&path).fs_context("failed to canonicalize path", path)?;
  if path.starts_with(scope) {
    std::fs::read(&path).fs_context("failed to read file", &path)
  } else {
    crate::error::bail!("forbidden path")
  }
}

fn watch<F: Fn() + Send + 'static>(dir: PathBuf, handler: F) {
  thread::spawn(move || {
    let (tx, rx) = std::sync::mpsc::channel();

    let mut watcher = notify_debouncer_full::new_debouncer(Duration::from_secs(1), None, tx)
      .expect("failed to start builtin server fs watcher");

    watcher
      .watch(&dir, notify::RecursiveMode::Recursive)
      .expect("builtin server failed to watch dir");

    loop {
      if let Ok(Ok(event)) = rx.recv() {
        if let Some(event) = event.first() {
          if !event.kind.is_access() {
            handler();
          }
        }
      }
    }
  });
}

#[cfg(test)]
mod tests {
  use super::resolve_asset;

  fn dist() -> std::path::PathBuf {
    let dir =
      std::env::temp_dir().join(format!("tauri-cli-dev-server-test-{}", std::process::id()));
    std::fs::create_dir_all(dir.join("docs")).unwrap();
    // the server canonicalizes the dist dir on startup (`start`); do the same
    // here so the scope check holds on platforms where the temp dir contains
    // symlinks (macOS `/var`) or short names (Windows)
    let dir = dunce::canonicalize(&dir).unwrap();
    std::fs::write(dir.join("index.html"), "<html>index</html>").unwrap();
    std::fs::write(dir.join("about.html"), "<html>about</html>").unwrap();
    std::fs::write(dir.join("docs/index.html"), "<html>docs</html>").unwrap();
    std::fs::write(dir.join("app.js"), "console.log('app')").unwrap();
    dir
  }

  #[test]
  fn resolves_assets_like_the_tauri_protocol() {
    let dir = dist();

    // exact hits
    assert_eq!(
      resolve_asset(&dir, "app.js").as_deref(),
      Some(b"console.log('app')" as &[u8])
    );
    // html fallbacks for extensionless paths
    assert_eq!(
      resolve_asset(&dir, "about").as_deref(),
      Some(b"<html>about</html>" as &[u8])
    );
    assert_eq!(
      resolve_asset(&dir, "docs").as_deref(),
      Some(b"<html>docs</html>" as &[u8])
    );
    assert_eq!(
      resolve_asset(&dir, "route").as_deref(),
      Some(b"<html>index</html>" as &[u8])
    );
    // missing subresources do not fall back
    assert_eq!(resolve_asset(&dir, "missing.js"), None);
    assert_eq!(resolve_asset(&dir, "assets/missing.png"), None);

    std::fs::remove_dir_all(dir).unwrap();
  }
}
