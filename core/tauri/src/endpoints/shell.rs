// Copyright 2019-2021 Tauri Programme within The Commons Conservancy
// SPDX-License-Identifier: Apache-2.0
// SPDX-License-Identifier: MIT

use crate::{endpoints::InvokeResponse, Params, Window};
use serde::Deserialize;

#[cfg(shell_execute)]
use std::sync::{Arc, Mutex};
use std::{collections::HashMap, path::PathBuf};

type ChildId = u32;
#[cfg(shell_execute)]
type ChildStore = Arc<Mutex<HashMap<ChildId, crate::api::command::CommandChild>>>;

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

fn default_env() -> Option<HashMap<String, String>> {
  Some(Default::default())
}

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
#[derive(Deserialize)]
#[serde(tag = "cmd", rename_all = "camelCase")]
pub enum Cmd {
  /// The execute script API.
  #[serde(rename_all = "camelCase")]
  Execute {
    program: String,
    args: Vec<String>,
    on_event_fn: String,
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
  pub fn run<M: Params>(self, window: Window<M>) -> crate::Result<InvokeResponse> {
    match self {
      Self::Execute {
        program,
        args,
        on_event_fn,
        options,
      } => {
        #[cfg(shell_execute)]
        {
          let mut command = if options.sidecar {
            crate::api::command::Command::new_sidecar(program)?
          } else {
            crate::api::command::Command::new(program)
          };
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
              if matches!(event, crate::api::command::CommandEvent::Terminated(_)) {
                command_childs().lock().unwrap().remove(&pid);
              }
              let js = crate::api::rpc::format_callback(on_event_fn.clone(), &event)
                .expect("unable to serialize CommandEvent");

              let _ = window.eval(js.as_str());
            }
          });

          Ok(pid.into())
        }
        #[cfg(not(shell_execute))]
        Err(crate::Error::ApiNotAllowlisted(
          "shell > execute".to_string(),
        ))
      }
      Self::KillChild { pid } => {
        #[cfg(shell_execute)]
        {
          if let Some(child) = command_childs().lock().unwrap().remove(&pid) {
            child.kill()?;
          }
          Ok(().into())
        }
        #[cfg(not(shell_execute))]
        Err(crate::Error::ApiNotAllowlisted(
          "shell > execute".to_string(),
        ))
      }
      Self::StdinWrite { pid, buffer } => {
        #[cfg(shell_execute)]
        {
          if let Some(child) = command_childs().lock().unwrap().get_mut(&pid) {
            match buffer {
              Buffer::Text(t) => child.write(t.as_bytes())?,
              Buffer::Raw(r) => child.write(&r)?,
            }
          }
          Ok(().into())
        }
        #[cfg(not(shell_execute))]
        Err(crate::Error::ApiNotAllowlisted(
          "shell > execute".to_string(),
        ))
      }
      Self::Open { path, with } => {
        #[cfg(shell_open)]
        match crate::api::shell::open(path, with) {
          Ok(_) => Ok(().into()),
          Err(err) => Err(crate::Error::FailedToExecuteApi(err)),
        }

        #[cfg(not(shell_open))]
        Err(crate::Error::ApiNotAllowlisted("shell > open".to_string()))
      }
    }
  }
}
