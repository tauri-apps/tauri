// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use axum::{
  extract::{State, WebSocketUpgrade, ws},
  http::{HeaderMap, HeaderValue, StatusCode, Uri, header, uri::Authority},
  response::{IntoResponse, Response},
};
use std::{
  net::{IpAddr, SocketAddr},
  path::{Path, PathBuf},
  thread,
  time::Duration,
};
use tauri_utils::mime_type::MimeType;
use tokio::sync::broadcast::{Sender, channel};

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

async fn handler(uri: Uri, headers: HeaderMap, state: State<ServerState>) -> impl IntoResponse {
  if request_host(&headers, &uri).is_none_or(|host| !is_allowed_host(&host.0, &state.address)) {
    return forbidden();
  }

  // Frontend files should not contain query parameters. This seems to be how Vite handles it.
  let uri = uri.path();

  let uri = if uri == "/" {
    uri
  } else {
    uri.strip_prefix('/').unwrap_or(uri)
  };

  let bytes = fs_read_scoped(state.dir.join(uri), &state.dir)
    .or_else(|_| fs_read_scoped(state.dir.join(format!("{uri}.html")), &state.dir))
    .or_else(|_| fs_read_scoped(state.dir.join(format!("{uri}/index.html")), &state.dir))
    .or_else(|_| fs_read_scoped(state.dir.join("index.html"), &state.dir));

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

async fn ws_handler(
  ws: WebSocketUpgrade,
  uri: Uri,
  headers: HeaderMap,
  state: State<ServerState>,
) -> Response {
  let Some(host) = request_host(&headers, &uri) else {
    return forbidden().into_response();
  };
  if !is_allowed_host(&host.0, &state.address)
    || !is_allowed_origin(headers.get(header::ORIGIN), &host)
  {
    return forbidden().into_response();
  }

  ws.on_upgrade(move |mut ws| async move {
    let mut rx = state.tx.subscribe();
    loop {
      tokio::select! {
        msg = ws.recv() => match msg {
          // the client disconnected
          None | Some(Err(_)) | Some(Ok(ws::Message::Close(_))) => break,
          // ignore any other client message, pings are answered automatically
          Some(Ok(_)) => {}
        },
        fs_reload_event = rx.recv() => {
          if fs_reload_event.is_err() {
            break;
          }
          let msg = ws::Message::Text(r#"{"reload": true}"#.into());
          if ws.send(msg).await.is_err() {
            break;
          }
        }
      }
    }
  })
}

fn forbidden() -> (StatusCode, [(header::HeaderName, String); 1], Vec<u8>) {
  (
    StatusCode::FORBIDDEN,
    [(header::CONTENT_TYPE, "text/plain".into())],
    vec![],
  )
}

/// Hostname (lowercase, without IPv6 brackets) and port of the request,
/// read from the `Host` header or the request URI authority (HTTP/2).
fn request_host(headers: &HeaderMap, uri: &Uri) -> Option<(String, Option<u16>)> {
  match headers.get(header::HOST) {
    Some(host) => parse_authority(host.to_str().ok()?.parse().ok()?),
    None => parse_authority(uri.authority()?.clone()),
  }
}

fn parse_authority(authority: Authority) -> Option<(String, Option<u16>)> {
  let host = authority.host();
  let host = host
    .strip_prefix('[')
    .and_then(|h| h.strip_suffix(']'))
    .unwrap_or(host);
  if host.is_empty() {
    return None;
  }
  Some((host.to_ascii_lowercase(), authority.port_u16()))
}

/// Only accept requests addressed to the server itself to prevent DNS rebinding attacks.
fn is_allowed_host(host: &str, address: &SocketAddr) -> bool {
  // `localhost` and its subdomains always resolve to the loopback interface
  // (`tauri.localhost` is used by the dev proxy on Android and Windows)
  if host == "localhost" || host.ends_with(".localhost") {
    return true;
  }
  match host.parse::<IpAddr>() {
    // an IP address can't be rebound
    Ok(ip) => ip.is_loopback() || ip == address.ip() || address.ip().is_unspecified(),
    Err(_) => false,
  }
}

/// Reject cross-site WebSocket connections.
fn is_allowed_origin(origin: Option<&HeaderValue>, host: &(String, Option<u16>)) -> bool {
  let Some(origin) = origin else {
    return true;
  };
  let Some(origin) = origin.to_str().ok().and_then(|o| url::Url::parse(o).ok()) else {
    return false;
  };
  let Some(origin_host) = origin.host_str() else {
    return false;
  };
  let origin_host = origin_host
    .strip_prefix('[')
    .and_then(|h| h.strip_suffix(']'))
    .unwrap_or(origin_host)
    .to_ascii_lowercase();

  // Tauri pages served through the dev proxy on mobile (`tauri://localhost` or `http(s)://tauri.localhost`)
  if origin.scheme() == "tauri" || origin_host == "tauri.localhost" {
    return true;
  }

  origin_host == host.0 && origin.port_or_known_default() == Some(host.1.unwrap_or(80))
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
      if let Ok(Ok(event)) = rx.recv()
        && let Some(event) = event.first()
        && !event.kind.is_access()
      {
        handler();
      }
    }
  });
}

#[cfg(test)]
mod tests {
  use super::*;

  fn host_of(host: &str) -> Option<(String, Option<u16>)> {
    let mut headers = HeaderMap::new();
    headers.insert(header::HOST, HeaderValue::from_str(host).unwrap());
    request_host(&headers, &Uri::from_static("/"))
  }

  fn allowed(host: &str, address: &str) -> bool {
    let address: SocketAddr = address.parse().unwrap();
    host_of(host).is_some_and(|host| is_allowed_host(&host.0, &address))
  }

  #[test]
  fn parses_host_header() {
    assert_eq!(
      host_of("127.0.0.1:1430"),
      Some(("127.0.0.1".into(), Some(1430)))
    );
    assert_eq!(host_of("[::1]:1430"), Some(("::1".into(), Some(1430))));
    assert_eq!(host_of("LocalHost"), Some(("localhost".into(), None)));
    assert_eq!(host_of(""), None);
    assert_eq!(
      request_host(&HeaderMap::new(), &Uri::from_static("/")),
      None
    );
    assert_eq!(
      request_host(
        &HeaderMap::new(),
        &Uri::from_static("http://127.0.0.1:1430/")
      ),
      Some(("127.0.0.1".into(), Some(1430)))
    );
  }

  #[test]
  fn host_check() {
    let local = "127.0.0.1:1430";
    assert!(allowed("127.0.0.1:1430", local));
    assert!(allowed("localhost:1430", local));
    assert!(allowed("[::1]:1430", local));
    assert!(allowed("tauri.localhost", local));
    assert!(!allowed("evil.com:1430", local));
    assert!(!allowed("localhost.evil.com", local));
    assert!(!allowed("192.168.1.10:1430", local));

    // mobile dev binds to the LAN address
    let lan = "192.168.1.10:1430";
    assert!(allowed("192.168.1.10:1430", lan));
    assert!(allowed("127.0.0.1:1430", lan));
    assert!(!allowed("192.168.1.11:1430", lan));
    assert!(!allowed("evil.com:1430", lan));

    // the unspecified address accepts any IP but no domain
    let any = "0.0.0.0:1430";
    assert!(allowed("192.168.1.11:1430", any));
    assert!(!allowed("evil.com:1430", any));
  }

  #[test]
  fn origin_check() {
    let host = ("127.0.0.1".to_string(), Some(1430));
    let check =
      |origin: &str| is_allowed_origin(Some(&HeaderValue::from_str(origin).unwrap()), &host);

    assert!(is_allowed_origin(None, &host));
    assert!(check("http://127.0.0.1:1430"));
    assert!(check("tauri://localhost"));
    assert!(check("http://tauri.localhost"));
    assert!(check("https://tauri.localhost"));
    assert!(!check("http://127.0.0.1:3000"));
    assert!(!check("http://evil.com"));
    assert!(!check("null"));

    let ipv6 = ("::1".to_string(), Some(1430));
    assert!(is_allowed_origin(
      Some(&HeaderValue::from_static("http://[::1]:1430")),
      &ipv6
    ));

    let default_port = ("localhost".to_string(), None);
    assert!(is_allowed_origin(
      Some(&HeaderValue::from_static("http://localhost")),
      &default_port
    ));
  }

  #[test]
  fn fs_read_scoped_rejects_outside_scope() {
    let dir = tempfile::tempdir().unwrap();
    let scope = dunce::canonicalize(dir.path()).unwrap();
    let inner = scope.join("dist");
    std::fs::create_dir(&inner).unwrap();
    std::fs::write(inner.join("index.html"), "inner").unwrap();
    std::fs::write(scope.join("secret.txt"), "secret").unwrap();

    assert_eq!(
      fs_read_scoped(inner.join("index.html"), &inner).unwrap(),
      b"inner"
    );
    assert!(fs_read_scoped(inner.join("../secret.txt"), &inner).is_err());
  }
}
