// Copyright 2019-2024 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

//! Scenarios of the Tauri core whose instructions are counted by the benchmark runner.
//!
//! Timings taken on shared CI machines move by several percent between two runs of the same
//! code. The number of instructions a piece of code executes does not, so it can tell a small
//! regression from noise. The runner executes each scenario under Valgrind's callgrind,
//! which only counts what runs inside [`tauri_bench_measure`]: the setup is free.
//!
//! None of this needs a webview: the IPC scenarios use the mock runtime.
//!
//! Usage: `bench_micro list` or `bench_micro <scenario>`.
//!
//! ***_Internal use only_***

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use tauri::{
  WebviewWindowBuilder,
  ipc::{CallbackFn, InvokeBody, Origin, RuntimeAuthority},
  test::{INVOKE_KEY, MockRuntime, get_ipc_response, mock_builder, mock_context, noop_assets},
  webview::InvokeRequest,
};
use tauri_utils::{
  acl::{
    Commands, Permission, PermissionSet, capability::Capability, manifest::Manifest,
    resolved::Resolved,
  },
  config::{Config, DisabledCspModificationKind},
  pattern::isolation::{Keys, RawIsolationPayload},
  platform::Target,
};

/// The function callgrind counts the instructions of.
/// Its name is what the runner passes to `--toggle-collect`, so it must not be mangled or inlined.
#[unsafe(no_mangle)]
#[inline(never)]
pub fn tauri_bench_measure(run: &mut dyn FnMut()) {
  run()
}

/// The code to measure, returned by the setup of a scenario.
type Measured = Box<dyn FnMut()>;
/// Prepares a scenario. Not measured.
type Setup = fn() -> Measured;

const PLUGINS: usize = 10;
const COMMANDS: usize = 30;

/// The manifests of a few plugins and a capability enabling them,
/// the size of what an application with a handful of plugins has.
fn acl() -> (BTreeMap<String, Manifest>, BTreeMap<String, Capability>) {
  let mut manifests = BTreeMap::new();
  let mut permissions = Vec::new();

  for plugin in 0..PLUGINS {
    let mut manifest = Manifest::default();
    let mut default_permissions = Vec::new();

    for command in 0..COMMANDS {
      for (kind, allow) in [("allow", true), ("deny", false)] {
        let identifier = format!("{kind}-command-{command}");
        let commands = vec![format!("command_{command}")];
        manifest.permissions.insert(
          identifier.clone(),
          Permission {
            identifier: identifier.clone(),
            commands: if allow {
              Commands {
                allow: commands,
                deny: Vec::new(),
              }
            } else {
              Commands {
                allow: Vec::new(),
                deny: commands,
              }
            },
            ..Default::default()
          },
        );
        if allow {
          default_permissions.push(identifier);
        }
      }
    }

    manifest.default_permission = Some(PermissionSet {
      identifier: "default".into(),
      description: "default permissions".into(),
      permissions: default_permissions,
    });
    manifests.insert(format!("plugin-{plugin}"), manifest);
    permissions.push(format!("plugin-{plugin}:default"));
  }

  let capability: Capability = serde_json::from_value(serde_json::json!({
    "identifier": "main",
    "windows": ["main"],
    "permissions": permissions,
  }))
  .expect("invalid capability");

  (manifests, BTreeMap::from([("main".into(), capability)]))
}

/// Parsing the configuration file of the API example.
fn config_parse() -> Measured {
  let config = include_str!("../../../examples/api/src-tauri/tauri.conf.json");
  Box::new(move || {
    let config: Config = serde_json::from_str(config).expect("invalid config");
    std::hint::black_box(config);
  })
}

/// Resolving the access control list, done when an app is compiled
/// and at runtime when a capability is added.
fn acl_resolve() -> Measured {
  let (manifests, capabilities) = acl();
  Box::new(move || {
    let resolved = Resolved::resolve(&manifests, capabilities.clone(), Target::Linux)
      .expect("failed to resolve the ACL");
    std::hint::black_box(resolved);
  })
}

/// The access control check every plugin command goes through, 1000 times.
fn acl_check() -> Measured {
  let (manifests, capabilities) = acl();
  let resolved =
    Resolved::resolve(&manifests, capabilities, Target::Linux).expect("failed to resolve the ACL");
  let authority = RuntimeAuthority::new(manifests, resolved);
  let command = format!("plugin:plugin-{}|command_{}", PLUGINS / 2, COMMANDS / 2);
  assert!(
    authority
      .resolve_access(&command, "main", "main", &Origin::Local)
      .is_some()
  );

  Box::new(move || {
    for _ in 0..1000 {
      let access = authority.resolve_access(&command, "main", "main", &Origin::Local);
      std::hint::black_box(access);
    }
  })
}

#[derive(Clone, Serialize, Deserialize)]
struct Record {
  id: u32,
  name: String,
  active: bool,
  score: f64,
  tags: Vec<String>,
}

#[tauri::command]
fn noop() {}

#[tauri::command]
fn echo(records: Vec<Record>) -> Vec<Record> {
  records
}

fn ipc_app() -> (tauri::App<MockRuntime>, tauri::WebviewWindow<MockRuntime>) {
  let app = mock_builder()
    .invoke_handler(tauri::generate_handler![noop, echo])
    .build(mock_context(noop_assets()))
    .expect("failed to build app");
  let webview = WebviewWindowBuilder::new(&app, "main", Default::default())
    .build()
    .expect("failed to create webview");
  (app, webview)
}

fn invoke_request(cmd: &str, body: InvokeBody) -> InvokeRequest {
  InvokeRequest {
    cmd: cmd.into(),
    callback: CallbackFn(0),
    error: CallbackFn(1),
    url: "tauri://localhost".parse().unwrap(),
    body,
    headers: Default::default(),
    invoke_key: INVOKE_KEY.to_string(),
  }
}

/// The Rust side of 100 IPC calls of a command that does nothing:
/// everything between the message of the webview and its response.
fn ipc_invoke() -> Measured {
  let (app, webview) = ipc_app();
  Box::new(move || {
    let _app = &app;
    for _ in 0..100 {
      let response = get_ipc_response(&webview, invoke_request("noop", InvokeBody::default()));
      assert!(response.is_ok());
    }
  })
}

/// An IPC call that deserializes and serializes around 100 KB of JSON.
fn ipc_invoke_json() -> Measured {
  let (app, webview) = ipc_app();
  let records: Vec<Record> = (0..1000u32)
    .map(|id| Record {
      id,
      name: format!("record number {id}"),
      active: id.is_multiple_of(2),
      score: id as f64 / 3.,
      tags: (0..3).map(|tag| format!("tag-{id}-{tag}")).collect(),
    })
    .collect();
  let body = serde_json::json!({ "records": records });

  Box::new(move || {
    let _app = &app;
    let response = get_ipc_response(
      &webview,
      invoke_request("echo", InvokeBody::Json(body.clone())),
    );
    assert!(response.is_ok());
  })
}

/// Decrypting a 64 KB message of the isolation pattern.
fn isolation_decrypt() -> Measured {
  let keys = Keys::new().expect("failed to generate the isolation keys");
  let nonce = [7u8; 12];
  let message = vec![42u8; 64 * 1024];
  let payload = keys
    .aes_gcm()
    .encrypt(&nonce, &message)
    .expect("failed to encrypt");
  let raw = serde_json::to_vec(&serde_json::json!({
    "nonce": nonce,
    "payload": payload,
    "contentType": "application/json",
  }))
  .unwrap();

  Box::new(move || {
    let payload = RawIsolationPayload::try_from(&raw).expect("invalid payload");
    let decrypted = keys.decrypt(payload).expect("failed to decrypt");
    assert_eq!(decrypted.len(), 64 * 1024);
  })
}

/// What is done to each HTML file when the assets are embedded:
/// parsing it, injecting the CSP nonces and meta tag, and serializing it again.
fn html_csp() -> Measured {
  let mut html = String::from(
    "<!doctype html><html><head><title>bench</title><style>body { margin: 0 }</style></head><body>",
  );
  for index in 0..200 {
    html.push_str(&format!(
      "<div class=\"row\" id=\"row-{index}\"><span>item {index}</span><script src=\"http://localhost/script-{index}.js\"></script></div>"
    ));
  }
  html.push_str("</body></html>");

  Box::new(move || {
    let document = tauri_utils::html::parse(html.clone());
    tauri_utils::html::inject_nonce_token(&document, &DisabledCspModificationKind::Flag(false));
    tauri_utils::html::inject_csp(&document, "default-src 'self'; script-src 'self'");
    let serialized = tauri_utils::html::serialize_node(&document);
    std::hint::black_box(serialized);
  })
}

/// Every scenario and its setup.
const SCENARIOS: &[(&str, Setup)] = &[
  ("config_parse", config_parse),
  ("acl_resolve", acl_resolve),
  ("acl_check", acl_check),
  ("ipc_invoke", ipc_invoke),
  ("ipc_invoke_json", ipc_invoke_json),
  ("isolation_decrypt", isolation_decrypt),
  ("html_csp", html_csp),
];

fn main() {
  let name = std::env::args().nth(1).unwrap_or_default();

  if name == "list" {
    for (name, _) in SCENARIOS {
      println!("{name}");
    }
    return;
  }

  let Some((_, setup)) = SCENARIOS.iter().find(|(scenario, _)| *scenario == name) else {
    eprintln!("unknown scenario `{name}`, run `bench_micro list`");
    std::process::exit(1);
  };

  let mut measured = setup();
  tauri_bench_measure(&mut measured);
}
