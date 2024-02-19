// Copyright 2019-2023 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use axum::{
  extract::{ws::WebSocket, WebSocketUpgrade},
  http::{header::CONTENT_TYPE, StatusCode, Uri},
  response::IntoResponse,
  routing::get,
  Router, Server,
};
use html5ever::{namespace_url, ns, LocalName, QualName};
use kuchiki::{traits::TendrilSink, NodeRef};
use notify::RecursiveMode;
use notify_debouncer_mini::new_debouncer;
use std::{
  net::{IpAddr, SocketAddr},
  path::{Path, PathBuf},
  sync::{mpsc::sync_channel, Arc},
  thread,
  time::Duration,
};
use tauri_utils::mime_type::MimeType;
use tokio::sync::broadcast::{channel, Sender};

const AUTO_RELOAD_SCRIPT: &str = include_str!("./auto-reload.js");

struct State {
  serve_dir: PathBuf,
  address: SocketAddr,
  tx: Sender<()>,
}

pub fn start<P: AsRef<Path>>(path: P, ip: IpAddr, port: Option<u16>) -> crate::Result<SocketAddr> {
  let serve_dir = path.as_ref().to_path_buf();

  let (server_url_tx, server_url_rx) = std::sync::mpsc::channel();

  std::thread::spawn(move || {
    tokio::runtime::Builder::new_current_thread()
      .enable_io()
      .build()
      .unwrap()
      .block_on(async move {
        let (tx, _) = channel(1);

        let tokio_tx = tx.clone();
        let serve_dir_ = serve_dir.clone();
        thread::spawn(move || {
          let (tx, rx) = sync_channel(1);
          let mut watcher = new_debouncer(Duration::from_secs(1), move |r| {
            if let Ok(events) = r {
              tx.send(events).unwrap()
            }
          })
          .unwrap();

          watcher
            .watcher()
            .watch(&serve_dir_, RecursiveMode::Recursive)
            .unwrap();

          loop {
            if rx.recv().is_ok() {
              let _ = tokio_tx.send(());
            }
          }
        });

        let mut auto_port = false;
        let mut port = port.unwrap_or_else(|| {
          auto_port = true;
          1430
        });

        let (server, server_url) = loop {
          let server_url = SocketAddr::new(ip, port);
          let server = Server::try_bind(&server_url);

          if !auto_port {
            break (server, server_url);
          }

          if server.is_ok() {
            break (server, server_url);
          }

          port += 1;
        };

        let state = Arc::new(State {
          serve_dir,
          tx,
          address: server_url,
        });
        let state_ = state.clone();
        let router = Router::new()
          .fallback(move |uri| handler(uri, state_))
          .route(
            "/__tauri_cli",
            get(move |ws: WebSocketUpgrade| async move {
              ws.on_upgrade(|socket| async move { ws_handler(socket, state).await })
            }),
          );

        match server {
          Ok(server) => {
            server_url_tx.send(Ok(server_url)).unwrap();
            server.serve(router.into_make_service()).await.unwrap();
          }
          Err(e) => {
            server_url_tx
              .send(Err(anyhow::anyhow!(
                "failed to start development server on {server_url}: {e}"
              )))
              .unwrap();
          }
        }
      })
  });

  server_url_rx.recv().unwrap()
}

async fn handler(uri: Uri, state: Arc<State>) -> impl IntoResponse {
  // Frontend files should not contain query parameters. This seems to be how vite handles it.
  let uri = uri.path();

  let uri = if uri == "/" {
    uri
  } else {
    uri.strip_prefix('/').unwrap_or(uri)
  };

  let file = std::fs::read(state.serve_dir.join(uri))
    .or_else(|_| std::fs::read(state.serve_dir.join(format!("{}.html", &uri))))
    .or_else(|_| std::fs::read(state.serve_dir.join(format!("{}/index.html", &uri))))
    .or_else(|_| std::fs::read(state.serve_dir.join("index.html")));

  file
    .map(|mut f| {
      let mime_type = MimeType::parse_with_fallback(&f, uri, MimeType::OctetStream);
      if mime_type == MimeType::Html.to_string() {
        let mut document = kuchiki::parse_html().one(String::from_utf8_lossy(&f).into_owned());
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

        with_html_head(&mut document, |head| {
          let script_el =
            NodeRef::new_element(QualName::new(None, ns!(html), "script".into()), None);
          script_el.append(NodeRef::new_text(AUTO_RELOAD_SCRIPT.replace(
            "{{reload_url}}",
            &format!("ws://{}/__tauri_cli", state.address),
          )));
          head.prepend(script_el);
        });

        f = tauri_utils::html::serialize_node(&document);
      }

      (StatusCode::OK, [(CONTENT_TYPE, mime_type)], f)
    })
    .unwrap_or_else(|_| {
      (
        StatusCode::NOT_FOUND,
        [(CONTENT_TYPE, "text/plain".into())],
        vec![],
      )
    })
}

async fn ws_handler(mut ws: WebSocket, state: Arc<State>) {
  let mut rx = state.tx.subscribe();
  while tokio::select! {
      _ = ws.recv() => return,
      fs_reload_event = rx.recv() => fs_reload_event.is_ok(),
  } {
    let ws_send = ws.send(axum::extract::ws::Message::Text(
      r#"{"reload": true}"#.to_owned(),
    ));
    if ws_send.await.is_err() {
      break;
    }
  }
}
