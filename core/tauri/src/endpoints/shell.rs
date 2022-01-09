// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use super::InvokeContext;
use crate::{api::ipc::CallbackFn, Runtime};
use serde::Deserialize;
use tauri_macros::{module_command_handler, CommandModule};

#[cfg(shell_execute)]
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, path::PathBuf};

type ChildId = u32;
#[cfg(shell_execute)]
type ChildStore = Arc<Mutex<HashMap<ChildId, crate::api::process::CommandChild>>>;

#[cfg(shell_execute)]
fn command_childs() -> &'static ChildStore {
  use once_cell::sync::Lazy;
  static STORE: Lazy<ChildStore> = Lazy::new(Default::default);
  &STORE
}

#[derive(Deserialize)]
#[serde(untagged)]
pub enum Buffer {
  Text(String),
  Raw(Vec<u8>),
}

#[allow(clippy::unnecessary_wraps)]
fn default_env() -> Option<HashMap<String, String>> {
  Some(HashMap::default())
}

#[allow(dead_code)]
#[derive(Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandOptions {
  #[serde(default)]
  sidecar: bool,
  cwd: Option<PathBuf>,
  // by default we don't add any env variables to the spawned process
  // but the env is an `Option` so when it's `None` we clear the env.
  #[serde(default = "default_env")]
  env: Option<HashMap<String, String>>,
}

/// The API descriptor.
#[derive(Deserialize, CommandModule)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  /// The execute script API.
  #[serde(rename_all = "camelCase")]
  Execute {
    program: String,
    args: Vec<String>,
    on_event_fn: CallbackFn,
    #[serde(default)]
    options: CommandOptions,
  },
  StdinWrite {
    pid: ChildId,
    buffer: Buffer,
  },
  KillChild {
    pid: ChildId,
  },
  Open {
    path: String,
    with: Option<String>,
  },
}

impl Cmd {
  #[allow(unused_variables)]
  fn execute<R: Runtime>(
    context: InvokeContext<R>,
    program: String,
    args: Vec<String>,
    on_event_fn: CallbackFn,
    options: CommandOptions,
  ) -> crate::Result<ChildId> {
    let mut command = if options.sidecar {
      #[cfg(not(shell_sidecar))]
      return Err(crate::Error::ApiNotAllowlisted(
        "shell > sidecar".to_string(),
      ));
      #[cfg(shell_sidecar)]
      crate::api::process::Command::new_sidecar(program)?
    } else {
      #[cfg(not(shell_execute))]
      return Err(crate::Error::ApiNotAllowlisted(
        "shell > execute".to_string(),
      ));
      #[cfg(shell_execute)]
      crate::api::process::Command::new(program)
    };
    #[cfg(any(shell_execute, shell_sidecar))]
    {
      command = command.args(args);
      if let Some(cwd) = options.cwd {
        command = command.current_dir(cwd);
      }
      if let Some(env) = options.env {
        command = command.envs(env);
      } else {
        command = command.env_clear();
      }
      let (mut rx, child) = command.spawn()?;

      let pid = child.pid();
      command_childs().lock().unwrap().insert(pid, child);

      crate::async_runtime::spawn(async move {
        while let Some(event) = rx.recv().await {
          if matches!(event, crate::api::process::CommandEvent::Terminated(_)) {
            command_childs().lock().unwrap().remove(&pid);
          }
          let js = crate::api::ipc::format_callback(on_event_fn, &event)
            .expect("unable to serialize CommandEvent");

          let _ = context.window.eval(js.as_str());
        }
      });

      Ok(pid)
    }
  }

  #[module_command_handler(shell_execute, "shell > execute")]
  fn stdin_write<R: Runtime>(
    _context: InvokeContext<R>,
    pid: ChildId,
    buffer: Buffer,
  ) -> crate::Result<()> {
    if let Some(child) = command_childs().lock().unwrap().get_mut(&pid) {
      match buffer {
        Buffer::Text(t) => child.write(t.as_bytes())?,
        Buffer::Raw(r) => child.write(&r)?,
      }
    }
    Ok(())
  }

  #[module_command_handler(shell_execute, "shell > execute")]
  fn kill_child<R: Runtime>(_context: InvokeContext<R>, pid: ChildId) -> crate::Result<()> {
    if let Some(child) = command_childs().lock().unwrap().remove(&pid) {
      child.kill()?;
    }
    Ok(())
  }

  #[module_command_handler(shell_open, "shell > open")]
  fn open<R: Runtime>(
    _context: InvokeContext<R>,
    path: String,
    with: Option<String>,
  ) -> crate::Result<()> {
    match crate::api::shell::open(
      path,
      if let Some(w) = with {
        use std::str::FromStr;
        Some(crate::api::shell::Program::from_str(&w)?)
      } else {
        None
      },
    ) {
      Ok(_) => Ok(()),
      Err(err) => Err(crate::Error::FailedToExecuteApi(err)),
    }
  }
}
