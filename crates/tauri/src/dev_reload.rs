// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Experimental development-only reload of an opt-in business process.
//!
//! The WebView, Tauri runtime and JSON state remain in the host. A worker is
//! replaced only after a versioned handshake. See `examples/dev-reload`.
//! Calls block: use a command's blocking task, never the UI/event-loop thread.

use serde_json::{json, Value};
use std::{
  fs,
  io::{BufRead, BufReader, Read, Write},
  path::PathBuf,
  process::{Child, ChildStdin, Command, Stdio},
  sync::mpsc::{self, Receiver, SyncSender},
  time::Duration,
};

const MAX_MESSAGE: u64 = 8 * 1024 * 1024;

struct Worker {
  child: Child,
  requests: SyncSender<Vec<u8>>,
  responses: Receiver<Result<Value, String>>,
}

impl Worker {
  fn start(path: &str, timeout: Duration) -> Result<Self, String> {
    let mut child = Command::new(path)
      .stdin(Stdio::piped())
      .stdout(Stdio::piped())
      .stderr(Stdio::inherit())
      .spawn()
      .map_err(|e| format!("cannot start reload worker: {e}"))?;
    let mut stdin: ChildStdin = child.stdin.take().unwrap();
    let stdout = child.stdout.take().unwrap();
    let (tx, responses) = mpsc::sync_channel(1);
    std::thread::spawn(move || {
      let mut reader = BufReader::new(stdout);
      loop {
        let mut line = Vec::new();
        let response = match Read::by_ref(&mut reader)
          .take(MAX_MESSAGE + 1)
          .read_until(b'\n', &mut line)
        {
          Ok(0) => break,
          Ok(_) if line.len() as u64 > MAX_MESSAGE => Err("worker message too large".into()),
          Ok(_) => serde_json::from_slice(&line).map_err(|e| e.to_string()),
          Err(e) => Err(e.to_string()),
        };
        let failed = response.is_err();
        if tx.send(response).is_err() || failed {
          break;
        }
      }
    });
    let (requests, pending) = mpsc::sync_channel::<Vec<u8>>(1);
    std::thread::spawn(move || {
      while let Ok(bytes) = pending.recv() {
        if stdin.write_all(&bytes).and_then(|_| stdin.flush()).is_err() {
          break;
        }
      }
    });
    let worker = Self {
      child,
      requests,
      responses,
    };
    let hello = worker.receive(timeout)?;
    if hello != json!({ "protocol": 1 }) {
      return Err("unsupported reload worker protocol".into());
    }
    Ok(worker)
  }

  fn receive(&self, timeout: Duration) -> Result<Value, String> {
    self
      .responses
      .recv_timeout(timeout)
      .map_err(|e| format!("reload worker did not respond: {e}"))?
  }

  fn call(&mut self, request: &Value, timeout: Duration) -> Result<Value, String> {
    let mut bytes = serde_json::to_vec(request).map_err(|e| e.to_string())?;
    if bytes.len() as u64 >= MAX_MESSAGE {
      return Err("reload request too large".into());
    }
    bytes.push(b'\n');
    self.requests.try_send(bytes).map_err(|e| e.to_string())?;
    self.receive(timeout)
  }
}

impl Drop for Worker {
  fn drop(&mut self) {
    let _ = self.child.kill();
    let _ = self.child.wait();
  }
}

/// Own this behind a mutex in Tauri managed state. Only trusted local worker
/// executables may be published to the manifest. This is not an IPC sandbox.
pub struct ReloadHost {
  manifest: PathBuf,
  generation: String,
  worker: Option<Worker>,
  state: Value,
  timeout: Duration,
}

impl ReloadHost {
  /// `manifest` contains an absolute executable path, atomically published by
  /// the build watcher. `state` must be JSON and compatible across generations.
  pub fn new(manifest: impl Into<PathBuf>, state: Value, timeout: Duration) -> Self {
    Self {
      manifest: manifest.into(),
      generation: String::new(),
      worker: None,
      state,
      timeout,
    }
  }

  /// Dispatch a serialized request. State commits only after a valid response.
  /// Existing requests finish before the next generation is selected. External
  /// side effects are not transactional and calls are never automatically retried.
  pub fn call(&mut self, command: &str, payload: Value) -> Result<Value, String> {
    match fs::read_to_string(&self.manifest) {
      Ok(path) => {
        let path = path.trim();
        if !std::path::Path::new(path).is_absolute() {
          return Err("reload manifest must contain an absolute executable path".into());
        }
        if path != self.generation || self.worker.is_none() {
          match Worker::start(path, self.timeout) {
            Ok(worker) => {
              self.worker = Some(worker);
              path.clone_into(&mut self.generation);
            }
            Err(e) if self.worker.is_some() => {
              eprintln!("Reload rejected; keeping previous worker: {e}")
            }
            Err(e) => return Err(e),
          }
        }
      }
      Err(_) if self.worker.is_some() => {}
      Err(e) => return Err(format!("cannot read reload manifest: {e}")),
    }
    let request = json!({ "command": command, "payload": payload, "state": self.state });
    let response = self.worker.as_mut().unwrap().call(&request, self.timeout);
    match response {
      Ok(response) if response.get("state").is_some() && response.get("result").is_some() => {
        self.state = response["state"].clone();
        Ok(response["result"].clone())
      }
      response => {
        self.worker = None;
        Err(match response {
          Ok(_) => "worker response requires state and result".into(),
          Err(e) => e,
        })
      }
    }
  }
}
